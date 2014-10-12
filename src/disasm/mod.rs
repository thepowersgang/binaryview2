// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Disassembly core
use self::state::State;
use sortedlist::SortedList;	// Allows treating of collection types as sorted lists

mod common_instrs;
mod state;
mod microcode;
mod instruction;
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
		}
	}
	/// Count total instructions converted
	pub fn instr_count(&self) -> uint {
		self.instructions.len()
	}
	
	/// Run disassembly on the todo list
	pub fn convert_queue(&mut self) -> uint
	{
		let todo = ::std::mem::replace(&mut self.todo_list, Vec::new());
		debug!("todo_list = {}", todo);
		let ret = todo.len();
		for (addr,mode) in todo.into_iter()
		{
			self.convert_from(addr, mode);
		}
		ret
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

		let is_first_in_run = true;
		
		// Keep processing until either a terminal instruction is located (break)
		// or an already-processed instruction is hit (while cond)
		while pos.is_end() || !pos.next().contains(addr)
		{
			let mut instr = match self.cpu.disassemble(self.memory, addr, mode)
				{
				Ok(i) => i,
				Err(e) => {
					error!("Disassembly of {:#x} [mode={}] failed: {}", addr, mode, e);
					return ()
					},
				};
			
			if is_first_in_run {
				instr.set_target();
			}
			
			// Set common state on instruction
			// - Straight out of the disassembler, it is just a bare instruction
			instr.set_addr(addr, mode);
			debug!("> {}", instr);
			
			// Execute with minimal state
			self.cpu.prep_state(&mut state, addr, mode);
			state.run(&instr);
			
			// Get list of jump targets from instruction
			for item in state.todo_list().iter()
			{
				let mut p = todo.find_ins(|e| e.cmp(item));
				if p.is_end() || p.next() != item {
					p.insert(item.clone());
				}
			}
			
			let is_terminal = instr.is_terminal();
			addr += instr.len as u64;
			pos.insert(box instr);
			
			// If instruction is terminal, break out of loop
			if is_terminal {
				break;
			}
		}
		
		debug!("- Complete at IP={:#x}", addr);
	}
}

// vim: ft=rust
