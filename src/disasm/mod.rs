// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Disassembly core
use self::state::State;
use self::block::Block;
use sortedlist::SortedList;	// Allows treating of collection types as sorted lists
use std::collections::HashSet;

#[macro_use] mod common_instrs;
mod state;
mod microcode;
mod instruction;
mod block;
pub mod cpus;

trait CPU
{
	/// Return the number of CPU-defined registers
	fn num_regs(&self) -> uint;
	
	/// Disassemble a single instruction
	fn disassemble(&self, &::memory::MemoryState, u64, uint) -> Result<instruction::Instruction,()>;
	/// Prepare state for exection of an instruction at the specified address
	fn prep_state(&self, &mut state::State, u64, uint);
	
	//// Check the outcome of a condition code check
	//fn check_condition(&self, &mut state::State, u8) -> ValueBool;
}

pub struct Disassembled<'a>
{
	memory: &'a ::memory::MemoryState,
	cpu: &'a CPU+'a,
	instructions: Vec<Box<instruction::Instruction>>,
	
	todo_list: Vec<(u64,uint)>,
	// TODO: Store is_call flag
	method_list: HashSet<(u64,uint)>,
}

impl<'a> Disassembled<'a>
{
	pub fn new<'s>(mem: &'s ::memory::MemoryState, cpu: &'s CPU) -> Disassembled<'s>
	{
		Disassembled {
			memory: mem,
			cpu: cpu,
			instructions: Vec::new(),
			todo_list: Vec::new(),
			method_list: HashSet::new(),
		}
	}
	/// Count total instructions converted
	pub fn instr_count(&self) -> uint {
		self.instructions.len()
	}
	
	pub fn dump(&self, f: &mut ::std::fmt::FormatWriter) -> Result<(),::std::fmt::FormatError>
	{
		for instr in self.instructions.iter()
		{
			if instr.is_call_target()
			{
				try!(write!(f, "\n"));
				try!(write!(f, "@"));
			}
			else if instr.is_target()
			{
				try!(write!(f, ">"));
			}
			else
			{
				try!(write!(f, " "));
			}
			try!(write!(f, "{}\n", instr));
		}
		Ok( () )
	}
	
	/// Run disassembly on the todo list
	pub fn convert_queue(&mut self) -> uint
	{
		let todo = ::std::mem::replace(&mut self.todo_list, Vec::new());
		info!("convert_queue(): todo = {}", todo);
		let ret = todo.len();
		for (addr,mode) in todo.into_iter()
		{
			self.convert_from(addr, mode);
		}
		ret
	}

	/// "Blockify" Pass
	///
	/// Breaks the code into blocks, separated by jump instructions and jump targets
	/// Also handles marking of instructions as call targets for later passes	
	pub fn pass_blockify(&mut self) -> uint
	{
		info!("pass_blockify()");
		let mut count = 0u;
		let mut state = State::null(self.cpu, self.memory);
		let mut block = Block::new_rc(0,0);
		
		// 1. Iterate all instructions
		for instr in self.instructions.iter_mut()
		{
			// (side) Mark call targets using global method list
			if self.method_list.contains( &(instr.addr(), instr.mode()) )
			{
				instr.set_call_target();
			}
			
			if instr.block().is_some()
			{
				// Skip, already assigned to a block
			}
			else
			{
				// Run instruction
				state.run(&**instr);
				
				// Flag call targets (Secondary job)
				// - Collate them 
				let mut was_jump = false;
				for &(_, iscall) in state.todo_list().iter()
				{
					if iscall {
					}
					else {
						was_jump = true;
					}
				}
				
				// If any of
				// - The instruction is terminal
				// - The instruction is a jump target
				// - or, the todo list contains a non-call entry
				// Terminate this block and create a new one
				// NOTE: is_terminal check not needed, all terminals will be
				//       followed by a target (or be at the end).
				if instr.is_target() || was_jump
				{
					debug!("New block triggered at {}:{:#x}", instr.mode(), instr.addr());
					count += 1;
					
					// TODO: Store state at end of the block
					// - Need to save state values, actual state contains references
					//block.set_state( state );
					
					// New block
					let newblock = Block::new_rc(instr.mode(), instr.addr());
					state = State::null(self.cpu, self.memory);
					
					block = newblock;
				}
				else
				{
					state.clear_todo_list();
				}
				
				// 3. Assign a code block to all instructions
				instr.set_block(block.clone());
			}
		}
	
		assert!( ::std::rc::is_unique(&block) || count > 0 );
		
		count
	}
	
	/// Determine the calling convention for methods
	pub fn pass_callingconv(&mut self) -> uint
	{
		// For all methods
		
		// - Create a state with all registers primed with Canary values
		// - Execute (branching state at conditional/multitarget jumps)
		// - When end of method is hit, save state.
		// - Spot reverse jumps and (TODO) [Run until stable] [Stop]
		0
	}
	
	/// Disassemble starting from a given address
	pub fn convert_from(&mut self, addr: u64, mode: uint)
	{
		debug!("convert_from(addr={:#x},mode={})", addr, mode);
		let mut todo = Vec::<(u64,uint)>::new();
		
		// Actual disassembly call
		self.convert_from_inner(addr, mode, &mut todo);
		
		// Disassembly pass (holds a mutable handle to the instruction list
		// Convert local todo list into the 'global' list (pruning duplicate
		// entries and already-converted entries)
		debug!("- TODO = {}", todo);
		for item in todo.into_iter()
		{
			match self.instructions.as_slice().binary_search(|e| (e.base,e.mode).cmp(&item))
			{
			::std::slice::NotFound(_) => {
				let mut p = self.todo_list.find_ins(|e| e.cmp(&item));
				if p.is_end() || *p.next() != item { 
					p.insert( item );
				}
				},
			::std::slice::Found(i) => {
				self.instructions.get_mut(i).set_target();
				},
			}
		}
	}
	
	/// (internal) Does the actual disassembly
	///
	/// Holds a mutable handle to self.instructions, so can't be part of convert_from
	fn convert_from_inner(&mut self, mut addr: u64, mode: uint, todo: &mut Vec<(u64,uint)>)
	{
		let mut state = State::null(self.cpu, self.memory);
		
		// Locate the insert location for the first instruction
		let mut pos = self.instructions.find_ins(|e| e.base.cmp(&addr));
		if !pos.is_end() && pos.next().contains(addr)
		{
			debug!("- Address {:#x},mode={} already processed", addr, mode);
			return ;
		}

		let mut is_first_in_run = true;
		
		// Keep processing until either a terminal instruction is located (break)
		// or an already-processed instruction is hit (while cond)
		while pos.is_end() || !pos.next().contains(addr)
		{
			let mut instr = match self.cpu.disassemble(self.memory, addr, mode)
				{
				Ok(i) => i,
				Err(e) => {
					error!("Disassembly of {:#x} [mode={}] failed: {}", addr, mode, e);
					// Return a placeholder, simplifying later code
					instruction::Instruction::invalid()
					},
				};
			
			if is_first_in_run {
				instr.set_target();
				is_first_in_run = false;
			}
			
			// Set common state on instruction
			// - Straight out of the disassembler, it is just a bare instruction
			instr.set_addr(addr, mode);
			debug!("> {}", instr);
			
			// Execute with minimal state
			self.cpu.prep_state(&mut state, addr, mode);
			state.run(&instr);
			
			let is_terminal = instr.is_terminal();
			addr += instr.len as u64;
			pos.insert(box instr);
			
			// If instruction is terminal, break out of loop
			if is_terminal {
				break;
			}
		}
		
		// Get list of jump targets from instruction
		for &(addr,iscall) in state.todo_list().iter()
		{
			let mut p = todo.find_ins(|e| e.cmp(&addr));
			if p.is_end() || *p.next() != addr {
				p.insert(addr.clone());
			}
			if iscall {
				self.method_list.insert( addr.clone() );
			}
		}
		
		debug!("- Complete at IP={:#x}", addr);
	}
}

// vim: ft=rust
