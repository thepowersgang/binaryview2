//
//
//
use std::rc::Rc;
use self::state::State;
use sortedlist::{SortedList,VecInsertPos};

pub mod cpus;
mod state;
mod microcode;

struct Instruction
{
	base: u64,
	len: u8,
	class: Rc<InstructionClass>,
	params: Vec<InstrParam>,
	
	is_target: bool,
}
enum InstrParam
{
	ParamTrueReg(u8),
	ParamTmpReg(u8),
	ParamImmediate(u64),
}

struct InstructionClass
{
	is_terminal: bool,
	print: fn(&mut ::std::fmt::Formatter, &[InstrParam]),
}

trait CPU
{
	/// Return the number of CPU-defined registers
	fn num_regs(&self) -> uint;
	
	/// Disassemble a single instruction
	fn disassemble(&self, &::memory::MemoryState, u64, uint) -> Result<Instruction,()>;
	/// Prepare state for exection of an instruction at the specified address
	fn prep_state(&self, &mut state::State, u64, uint);
}

struct Disassembled<'a>
{
	memory: &'a ::memory::MemoryState,
	cpu: &'a CPU+'a,
	instructions: Vec<Box<Instruction>>,
}

impl<'a> Disassembled<'a>
{
	fn is_done(&self, addr: u64) -> bool
	{
		false
	}
	
	fn convert_from(&mut self, mut addr: u64, mode: uint)
	{
		let mut pos = self.instructions.find_ins(|e| e.base.cmp(&addr));
		if !pos.is_end() && pos.next().contains(addr)
		{
			return ;
		}

		let is_first_in_run = true;
		
		while pos.is_end() || !pos.next().contains(addr)
		{
			let mut instr = match self.cpu.disassemble(self.memory, addr, mode)
				{
				Ok(i) => i,
				Err(_) => return (),
				};
			
			if is_first_in_run {
				instr.set_target();
			}
			
			// Set common state on instruction
			// - Straight out of the disassembler, it is just a bare instruction
			
			// Execute with minimal state
			let mut state = State::null(self.cpu, self.memory);
			self.cpu.prep_state(&mut state, addr, mode);
			state.run(&instr);
			
			// TODO: Get list of jump targets from instruction
			
			let is_terminal = instr.is_terminal();
			addr += instr.len as u64;
			pos.insert(box instr);
			
			// If instruction is terminal, break out of loop
			if is_terminal {
				break;
			}
		}
	}
}

impl Instruction
{
	fn set_target(&mut self) {
		self.is_target = true;
	} 
	
	fn contains(&self, addr: u64) -> bool
	{
		self.base <= addr && addr < self.base + self.len as u64
	}
	fn is_terminal(&self) -> bool {
		(*self.class).is_terminal
	}
}

// vim: ft=rust
