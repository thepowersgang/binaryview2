// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Disassembly core
use self::state::{State,RunMode};
use self::block::Block;
//use sortedlist::SortedList;	// Allows treating of collection types as sorted lists
use std::collections::{HashSet,HashMap};
use bit_set::BitSet;
use std::default::Default;
use value::Value;

#[macro_use] mod common_instrs;
mod state;
mod microcode;
mod instruction;
mod block;
pub mod cpus;

pub type CPUMode = u32;
#[derive(Copy,PartialEq,PartialOrd,Eq,Ord,Clone,Hash)]
pub struct CodePtr(CPUMode, u64);

#[derive(Copy,PartialEq,Eq,Clone,Hash)]
pub struct CodeRange(CodePtr, CodePtr);

pub trait CPU
{
	/// Return the number of CPU-defined registers
	fn num_regs(&self) -> u16;
	
	/// Disassemble a single instruction
	fn disassemble(&self, &::memory::MemoryState, u64, CPUMode) -> Result<instruction::Instruction,()>;
	/// Prepare state for exection of an instruction at the specified address
	fn prep_state(&self, &mut state::State, u64, CPUMode);
	
	//// Check the outcome of a condition code check
	//fn check_condition(&self, &mut state::State, u8) -> ValueBool;
}

pub struct Disassembled<'a>
{
	memory: &'a ::memory::MemoryState,
	cpu: &'a (CPU+'a),
	blocks: Vec<Box<Block>>,
	
	todo_list: HashSet<CodePtr>,
	// TODO: Store is_call flag
	method_list: HashMap<CodePtr,block::Function>,
}

impl<'a> Disassembled<'a>
{
	pub fn new<'s>(mem: &'s ::memory::MemoryState, cpu: &'s CPU) -> Disassembled<'s>
	{
		Disassembled {
			memory: mem,
			cpu: cpu,
			blocks: Vec::new(),
			todo_list: Default::default(),
			method_list: Default::default(),
		}
	}
	/// Count total instructions converted
	pub fn instr_count(&self) -> usize {
		self.blocks.iter().fold(0, |v,x| v + x.instrs().len())
	}
	
	fn find_block_for(&self, addr: CodePtr) -> Result<usize,usize>
	{
		self.blocks.binary_search_by(|e| e.range().contains_ord(addr))
	}
	
	// TODO: Should this be moved to being Debug or Display?
	pub fn dump(&self, f: &mut ::std::fmt::Write) -> ::std::fmt::Result
	{
		for block in self.blocks.iter()
		{
			if let Some(info) = self.method_list.get( &block.range().first() )
			{
				try!(write!(f, "\n"));
				try!(write!(f, "{}\n", info));
				try!(write!(f, "@"));
			}
			else
			{
				try!(write!(f, ">"));
			}
			for i in block.instrs().iter()
			{
				try!(write!(f, "{}\n ", i));
			}
			if let Some(end_state_data) = block.end_state()
			{
				try!(write!(f, "={}\n", end_state_data));
			}
		}
		Ok( () )
	}
	
	/// Run disassembly on the todo list
	pub fn convert_queue(&mut self) -> usize
	{
		info!("convert_queue(): todo = {:?}", self.todo_list);
		let mut ret = 0;
		while self.todo_list.len() > 0
		{
			let todo = ::std::mem::replace(&mut self.todo_list, HashSet::new());
			ret += todo.len();
			for ptr in todo.into_iter()
			{
				self.convert_from(ptr);
			}
		}
		ret
	}

	/// "Blockify" Pass
	///
	/// Breaks the code into blocks, separated by jump instructions and jump targets
	/// Also handles marking of instructions as call targets for later passes	
	pub fn pass_block_run(&mut self) -> usize
	{
		//info!("pass_blockify()");
		let mut count = 0;
		for block in self.blocks.iter_mut()
		{
			// Execute block
			if block.end_state().is_some()
			{
				trace!("Block {} already has state: {}", block.range(), block.end_state().unwrap());
				continue ;
			}
			
			let mut state = State::null(RunMode::Blockify, self.cpu, self.memory);
			for instr in block.instrs().iter()
			{
				state.run(&*instr);
				
				// Sanity check that jumps are the last instruction in the block
				let mut was_jump = false;
				for &(_, iscall) in state.todo_list().iter()
				{
					if iscall {
					}
					else {
						was_jump = true;
					}
				}
				assert!(was_jump);
			}
			
			count += 1;
			block.set_state( state.unwrap_data() );
			debug!("Block {}: New state {}", block.range(), block.end_state().unwrap());
		}
		count
	}
	
	/// Run a single function, determining what registers it uses and clobbers
	///
	/// Returns (will_be_fully_known, clobbers, inputs)
	fn pass_callingconv_runfcn(&self, addr: CodePtr) -> (bool, BitSet, BitSet)
	{
		let mut end_states = Vec::new();
		let mut will_be_fully_known = true;	// cleared if non-fully-known method is called
		
		// Open scope to properly end the borrows owned by callee_lookup
		{
			// A closure called by State::call() that handles the calling convention
			let mut callee_lookup = |state: &mut state::State, tgt_addr: CodePtr| {
				trace!("callee_lookup(tgt_addr={}), addr={}", tgt_addr, addr);
				if tgt_addr == addr {
					warn!("TODO: Handle direct recursion");
					return ;
				}
				// Locate method with this address
				match self.method_list.get(&tgt_addr)
				{
				Some(i) => match i.cc_state()
					{
					block::CCState::Unknown => {
						debug!("Method {} cc_state = Unknown", tgt_addr);
						// Do nothing, the function is unknown (hence we can't know anything about it)
						// - Flag currently caller as being partially known
						will_be_fully_known = false;
						},
					block::CCState::Partial => {
						debug!("Method {} cc_state = Partial", tgt_addr);
						// We have partial knowledge of the function's register actions
						// - Clobber clobbers and read from inputs
						for r in i.inputs().iter() {
							state.data_mut().read_reg(r as u8);
						}
						for r in i.clobbers().iter() {
							state.data_mut().write_reg(r as u8, Value::Unknown);
						}
						// - Flag callee as only being partially known
						will_be_fully_known = false;
						},
					block::CCState::Full => {
						debug!("Method {} cc_state = Full", tgt_addr);
						// Function's register actons are fully known, so apply inputs and clobbers
						// - DONT mark caller as partially known, as it's now fully known
						for r in i.inputs().iter() {
							state.data_mut().read_reg(r as u8);
						}
						for r in i.clobbers().iter() {
							state.data_mut().write_reg(r as u8, Value::Unknown);
						}
						},
					},
				None => {
					warn!("Calling unknown function {} in calling convention pass", addr);
					},
				}
				};
			
			// - Create a state with all registers primed with Canary values
			let init_state = {
				let mut state = State::null(RunMode::CallingConv, self.cpu, self.memory);
				state.fill_canary();
				//self.cpu.prep_method(&mut state);
				state.unwrap_data()
				};
			let block_idx = self.find_block_for(addr).ok().expect("Method code not disassembled");
			let mut stack = Vec::<(usize, state::StateData, Vec<usize>)>::new();
			stack.push( (block_idx, init_state, Vec::new()) );
			// - Execute (branching state at conditional/multitarget jumps)
			while let Some( (block_idx, data, history) ) = stack.pop()
			{
				let mut state = State::from_data(RunMode::CallingConv, self.memory, data, &mut callee_lookup);
				let block = &*self.blocks[block_idx];
				//  > Run block to completion off 'current' state
				for i in block.instrs()
				{
					state.run(i);
				}
				// - Spot reverse jumps and (TODO) [Run until stable] [Stop]
				//  > If only one target, push current state to stack (along with target)
				if block.refs().len() == 0
				{
					// - When end of method is hit, save state.
					trace!("- Reached end of method");
					end_states.push( state.unwrap_data() );
				}
				else if block.refs().len() == 1
				{
					let addr = block.refs()[0];
					trace!("- Only option is {}", addr);
					let mut newhist = history;
					newhist.push(block_idx);
					let block_idx = self.find_block_for(addr).ok().expect("Target block isn't disassembled");
					if newhist.contains(&block_idx)
					{
						trace!("- Loopback, ignoring (TODO: Handle)");
						continue ;
					}
					stack.push( (block_idx, state.unwrap_data(), newhist) );
				}
				//  > If multiples, clone state with branch condition
				else
				{
					let refs = block.refs();
					trace!("- Options are {:?}", refs);
					let mut newhist = history;
					newhist.push(block_idx);
					let data = state.unwrap_data();
					for &addr in &refs[..refs.len()-1]	// all but last
					{
						let next_block_idx = self.find_block_for(addr).ok().expect("Target block isn't disassembled");
						if newhist.contains(&next_block_idx)
						{
							trace!("- Loopback, ignoring (TODO: Handle)");
							continue ;
						}
						stack.push( (next_block_idx, data.clone(), newhist.clone()) );
					}
					
					let &addr = refs.last().unwrap();
					let next_block_idx = self.find_block_for(addr).ok().expect("Target block isn't disassembled");
					if newhist.contains(&next_block_idx)
					{
						trace!("- Loopback, ignoring (TODO: Handle)");
						continue ;
					}
					stack.push( (next_block_idx, data, newhist) );
				}
				//debug!("New Stack = {:?}", stack);
			}
		}
		// Collate end states
		debug!("end_states = {:?}", end_states);
		let mut clobbers = BitSet::default();
		let mut inputs = BitSet::default();
		for sd in end_states
		{
			trace!("Clobbers: {:?} |= {:?}", clobbers, sd.get_clobbers());
			clobbers.union_with( &sd.get_clobbers() );
			trace!("Inputs: {:?} |= {:?}", inputs, sd.get_inputs());
			inputs.union_with( &sd.get_inputs() );
		}
		
		(will_be_fully_known, clobbers, inputs)
	}
	
	/// Determine the calling convention for methods
	pub fn pass_callingconv(&mut self) -> usize
	{
		let mut ret = 0;

		// For all methods, run and locate clobbers/inputs
		// NOTE: pass_callingconv_runfcn requires access to self.method_list, so we can't lock it
		for addr in self.method_list.keys().map(|x| *x).collect::<Vec<_>>()
		{
			debug!("Method {}: info={:?}", addr, self.method_list[&addr]);
			
			let start_state = self.method_list[&addr].cc_state();
			
			// Return if cc_state is Full
			if start_state == block::CCState::Full {
				trace!("- Function fully known");
				continue ;
			}
			
	
			let (fully_known, clobbers, inputs) = self.pass_callingconv_runfcn(addr);
			self.method_list.get_mut(&addr).unwrap().set_reg_usage(fully_known, inputs, clobbers);
			
			// Only increment count if the state changed
			let new_state = self.method_list[&addr].cc_state();
			if new_state != start_state
			{
				assert!(new_state != block::CCState::Unknown);
				assert!(start_state != block::CCState::Full);
				ret += 1;
			}
		}
		
		ret
	}
	
	/// Disassemble starting from a given address
	pub fn convert_from(&mut self, ip: CodePtr)
	{
		debug!("convert_from(ip={})", ip);
		let mut todo = HashSet::<CodePtr>::new();
		
		if let Ok(i) = self.find_block_for(ip)
		{
			let range = self.blocks[i].range();
			if range.first() == ip
			{
				debug!("- {} already converted, start of block to {}", ip, range.last());
			}
			else
			{
				debug!("- {} already converted, stored in block '{}', breaking", ip, range);
				
				let newblock = Box::new( self.blocks[i].split_at(ip) );
				self.blocks.insert(i+1, newblock);
			}
			return ;
		}
		
		// Actual disassembly call
		let block = Box::new( self.convert_block(ip, &mut todo) );
		let i = match self.find_block_for(block.range().first())
			{
			Err(i) => i,
			Ok(_) => panic!("Block at address {} already converted", block.range())
			};
		self.blocks.insert(i, block);
		
		// Disassembly pass (holds a mutable handle to the instruction list
		// Convert local todo list into the 'global' list (pruning duplicate
		// entries and already-converted entries)
		debug!("- TODO = {:?}", todo);
		for item in todo.into_iter()
		{
			// Find a block that contains this instruction
			// - If found, split the block and tag the first instruction
			// - Otherwise, add to the global to-do list
			match self.find_block_for(item)
			{
			Err(i) => {
				if i > 0 {
					trace!("i = {}, block = {}", i, self.blocks[i-1].range());
					assert!( self.blocks[i-1].range().contains_ord(item) == ::std::cmp::Ordering::Less);
				}
				self.todo_list.insert( item );
				},
			Ok(i) => {
				if self.blocks[i].range().first() == item {
					// Equal, ignore
					trace!("{} is block {}, ignoring", item, i);
				}
				else {
					assert!( self.blocks[i].range().contains(item) );
					let newblock = Box::new( self.blocks[i].split_at(item) );
					self.blocks.insert(i+1, newblock);
				}
				},
			}
		}
	}
	
	/// (internal) Does the actual disassembly
	///
	/// Holds a mutable handle to self.instructions, so can't be part of convert_from
	fn convert_block(&mut self, start: CodePtr, todo: &mut HashSet<CodePtr>) -> Block
	{
		let mut state = State::null(RunMode::Parse, self.cpu, self.memory);
		let mut instructions = Vec::new(); 
		
		let mut link_to_next = true;
		
		let mut addr = start.addr();
		let mode = start.mode();
		
		// Keep processing until either a terminal instruction is located (break)
		// or an already-processed instruction is hit (while cond)
		loop
		{
			if instructions.len() > 0 && self.todo_list.contains( &CodePtr::new(mode, addr) )
			{
				trace!("- Hit target");
				break;
			}
			
			let mut instr = match self.cpu.disassemble(self.memory, addr, mode)
				{
				Ok(i) => i,
				Err(e) => {
					error!("Disassembly of {:#x} [mode={}] failed: {:?}", addr, mode, e);
					// Return a placeholder, simplifying later code
					instruction::Instruction::invalid()
					},
				};
			
			// Set common state on instruction
			// - Straight out of the disassembler, it is just a bare instruction
			instr.set_addr( CodePtr(mode, addr) );
			debug!("> {:?}", instr);
			
			// Instruction was conditional, stop the current block and run with this instruction in a separate block
			if instr.is_conditional() {
				todo.insert( CodePtr::new(mode, addr + instr.len as u64) );
			
				// If we're processing an instruction AFTER the first, break
				if ! instructions.is_empty() {
					// Leave link_to_next as true, it will lead to linking this block with the conditional
					todo.insert( CodePtr::new(mode, addr) );
					trace!("- Conditional, breaking");
					break;
				}
			}
			
			// Execute with minimal state
			self.cpu.prep_state(&mut state, addr, mode);
			state.run(&instr);
			
			let is_terminal = instr.is_terminal();
			
			addr += instr.len as u64;
			instructions.push(instr);
			
			// If instruction is terminal, break out of loop
			if is_terminal {
				link_to_next = false;
				break;
			}
			let found_tgt = self.find_block_for( CodePtr::new(mode, addr) ).is_ok();
			if found_tgt {
				todo.insert( CodePtr::new(mode, addr) );
				break;
			}
		}
		
		instructions[0].set_target();

		
		let mut refs = Vec::new();
		
		if link_to_next {
			refs.push( CodePtr::new(mode, addr) );
		}
		
		// Get list of jump targets from instruction
		for &(addr,iscall) in state.todo_list().iter()
		{
			todo.insert( addr.clone() );
			if iscall {
				self.method_list.insert( addr.clone(), Default::default() );
			}
			else {
				refs.push( addr.clone() );
			}
		}
		
		debug!("- Complete at IP={:#x}", addr);
		Block::new(instructions, refs)
	}
}

impl CodePtr
{
	pub fn new(mode: CPUMode, addr: u64) -> CodePtr
	{
		CodePtr(mode, addr)
	}
	
	pub fn mode(&self) -> CPUMode { self.0 }
	pub fn addr(&self) -> u64 { self.1 }
}

impl ::std::fmt::Display for CodePtr
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		write!(f, "{}:{:#08x}", self.0, self.1)
	}
}
impl ::std::fmt::Debug for CodePtr
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		write!(f, "{}:{:#x}", self.0, self.1)
	}
}

impl CodeRange
{
	pub fn new(first: CodePtr, last: CodePtr) -> CodeRange
	{
		CodeRange(first, last)
	}
	
	pub fn first(&self) -> CodePtr {
		return self.0
	}
	pub fn last(&self) -> CodePtr {
		return self.1
	}
	
	pub fn contains(&self, ptr: CodePtr) -> bool {
		self.contains_ord(ptr) == ::std::cmp::Ordering::Equal
	}
	pub fn contains_ord(&self, ptr: CodePtr) -> ::std::cmp::Ordering {
		use std::cmp::Ordering;
		match self.0.cmp(&ptr)
		{
		Ordering::Greater => Ordering::Greater,
		Ordering::Equal => Ordering::Equal,
		Ordering::Less => match self.1.cmp(&ptr)
			{
			Ordering::Greater => Ordering::Equal,
			Ordering::Equal => Ordering::Equal,
			Ordering::Less => Ordering::Less,
			}
		}
	}
}

impl ::std::fmt::Display for CodeRange
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		write!(f, "{}--{}", self.0, self.1)
	}
}

// vim: ft=rust
