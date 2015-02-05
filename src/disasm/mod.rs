// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Disassembly core
use self::state::{State,RunMode};
use self::block::Block;
use sortedlist::SortedList;	// Allows treating of collection types as sorted lists
use std::collections::HashSet;

#[macro_use] mod common_instrs;
mod state;
mod microcode;
mod instruction;
mod block;
pub mod cpus;

pub type CPUMode = u32;
#[derive(Copy,PartialEq,PartialOrd,Eq,Ord,Clone,Hash)]
pub struct CodePtr(CPUMode, u64);

trait CPU
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
	method_list: HashSet<CodePtr>,
}

impl<'a> Disassembled<'a>
{
	pub fn new<'s>(mem: &'s ::memory::MemoryState, cpu: &'s CPU) -> Disassembled<'s>
	{
		Disassembled {
			memory: mem,
			cpu: cpu,
			blocks: Vec::new(),
			todo_list: HashSet::new(),
			method_list: HashSet::new(),
		}
	}
	/// Count total instructions converted
	pub fn instr_count(&self) -> usize {
		self.blocks.iter().fold(0, |v,x| v + x.instrs().len())
	}
	
	// TODO: Should this be moved to being Debug or Display?
	pub fn dump(&self, f: &mut ::std::fmt::Writer) -> ::std::fmt::Result
	{
		for block in self.blocks.iter()
		{
			if self.method_list.contains( &block.first_addr() )
			{
				try!(write!(f, "\n"));
				try!(write!(f, "\n"));
				// TODO: Print method information (clobbers, outputs, etc)
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
				try!(write!(f, "{}\n", end_state_data));
			}
		}
		Ok( () )
	}
	
	/// Run disassembly on the todo list
	pub fn convert_queue(&mut self) -> usize
	{
		info!("convert_queue(): todo = {:?}", self.todo_list);
		let todo = ::std::mem::replace(&mut self.todo_list, HashSet::new());
		let ret = todo.len();
		for ptr in todo.into_iter()
		{
			self.convert_from(ptr);
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
			}
			
			count += 1;
			block.set_state( state.unwrap_data() );
		}
		count
	}
	
	/// Determine the calling convention for methods
	pub fn pass_callingconv(&mut self) -> usize
	{
		// For all methods
		//for instr in self.instructions.iter_mut()
		//{
		//	if ! instr.is_call_target() {
		//		continue ;
		//	}
		//	
		//	// - Create a state with all registers primed with Canary values
		//	let state = State::null(RunMode::CallingConv, self.cpu, self.memory);
		//	// - Execute (branching state at conditional/multitarget jumps)
		//	// - When end of method is hit, save state.
		//	// - Spot reverse jumps and (TODO) [Run until stable] [Stop]
		//}
		0
	}
	
	/// Disassemble starting from a given address
	pub fn convert_from(&mut self, ip: CodePtr)
	{
		debug!("convert_from(ip={})", ip);
		let mut todo = HashSet::<CodePtr>::new();
		
		if let Ok(i) = self.blocks.binary_search_by(|e| e.partial_cmp(&ip).unwrap())
		{
			debug!("- Already converted, stored in block '{}--{}'", self.blocks[i].first_addr(), self.blocks[i].last_addr());
			return ;
		}
		
		// Actual disassembly call
		let block = box self.convert_block(ip, &mut todo);
		let i = match self.blocks.binary_search_by(|e| e.partial_cmp(&block.first_addr()).unwrap())
			{
			Err(i) => i,
			Ok(_) => panic!("Block at address {} already converted", block.first_addr())
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
			match self.blocks.binary_search_by(|e| e.partial_cmp(&item).unwrap())
			{
			Err(i) => {
				if i > 0 {
					trace!("i = {}, block = {:?}", i, self.blocks[i-1].first_addr());
					assert!( self.blocks[i-1].first_addr() < item);
					assert!( self.blocks[i-1].last_addr() < item);
				}
				self.todo_list.insert( item );
				},
			Ok(i) => {
				if self.blocks[i].first_addr() == item {
					// Equal, ignore
					trace!("{} is block {}, ignoring", item, i);
				}
				else {
					assert!( self.blocks[i].first_addr() < item );
					assert!( self.blocks[i].last_addr() >= item );
					let newblock = box self.blocks[i].split_at(item);
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
			
			// Execute with minimal state
			self.cpu.prep_state(&mut state, addr, mode);
			state.run(&instr);
			
			let is_terminal = instr.is_terminal();
			let is_cnd = instr.is_conditional();
			addr += instr.len as u64;
			instructions.push(instr);
			
			// If instruction is terminal, break out of loop
			if is_terminal {
				break;
			}
			if is_cnd {
				todo.insert( CodePtr::new(mode, addr) );
				break;
			}
		}
		
		instructions[0].set_target();

		
		// Get list of jump targets from instruction
		for &(addr,iscall) in state.todo_list().iter()
		{
			todo.insert( addr.clone() );
			if iscall {
				self.method_list.insert( addr.clone() );
			}
		}
		
		debug!("- Complete at IP={:#x}", addr);
		Block::new(instructions)
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

// vim: ft=rust
