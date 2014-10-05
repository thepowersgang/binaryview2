//
//
//
use std::rc::Rc;
use self::state::State;
use sortedlist::SortedList;	// Allows treating of collection types as sorted lists

pub mod cpus;
mod common_instrs;
mod state;
mod microcode;

struct Instruction
{
	mode: uint,
	base: u64,
	len: u8,
	
	condition: u8,
	class: &'static InstructionClass + 'static,
	params: Vec<InstrParam>,
	
	is_target: bool,
}
#[deriving(PartialEq)]
enum InstrParam
{
	ParamTrueReg(u8),
	ParamTmpReg(u8),
	ParamImmediate(u64),
}

trait InstructionClass
{
	fn name(&self) -> &str;
	fn is_terminal(&self, &[InstrParam]) -> bool;
	fn print(&self, &mut ::std::fmt::Formatter, &[InstrParam]) -> Result<(),::std::fmt::FormatError>;
	fn forwards(&self, &mut State, &[InstrParam]);
	fn backwards(&self, &mut State, &[InstrParam]);
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

pub struct Disassembled<'a>
{
	memory: &'a ::memory::MemoryState,
	cpu: &'a CPU+'a,
	instructions: Vec<Box<Instruction>>,
	
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
	
	pub fn convert_queue(&mut self) -> uint
	{
		let todo = ::std::mem::replace(&mut self.todo_list, Vec::new());
		debug!("todo_list = {}", todo);
		// TODO: 
		let ret = todo.len();
		for (addr,mode) in todo.into_iter()
		{
			self.convert_from(addr, mode);
		}
		ret
	}
	
	pub fn convert_from(&mut self, mut addr: u64, mode: uint)
	{
		debug!("convert_from(addr={:#x},mode={})", addr, mode);
		let mut todo = Vec::<(u64,uint)>::new();
		
		{
			let mut pos = self.instructions.find_ins(|e| e.base.cmp(&addr));
			if !pos.is_end() && pos.next().contains(addr)
			{
				debug!("- Already processed");
				return ;
			}

			let is_first_in_run = true;
			
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
				instr.mode = mode;
				instr.base = addr;
				debug!("> {}", instr);
				
				// Execute with minimal state
				let mut state = State::null(self.cpu, self.memory);
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
		}
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
			_ => {},
			}
		}
		
		debug!("- Complete at IP={:#x}", addr);
		
	}
}

impl Instruction
{
	fn new(
		len: u8,
		condition: u8,
		class: &'static InstructionClass + 'static,
		params: Vec<InstrParam>
		) -> Instruction
	{
		Instruction {
			mode: 0,
			base: 0,
			len: len,
			condition: condition,
			class: class,
			params: params,
			is_target: false,
		}
	}
	fn set_target(&mut self) {
		self.is_target = true;
	} 
	
	fn contains(&self, addr: u64) -> bool
	{
		self.base <= addr && addr < self.base + self.len as u64
	}
	fn is_terminal(&self) -> bool {
		self.condition == 0xE && self.class.is_terminal(self.params.as_slice())
	}
}

impl ::std::fmt::Show for Instruction
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(),::std::fmt::FormatError>
	{
		try!( write!(f, "[{}:{:8x}]+{:u} {} ", self.mode, self.base, self.len, self.class.name()) );
		try!( self.class.print(f, self.params.as_slice()) );
		Ok( () )
	}
}

impl ::std::fmt::Show for InstrParam
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(),::std::fmt::FormatError>
	{
		match self
		{
		&ParamTrueReg(r) => write!(f, "R{}", r),
		&ParamTmpReg(r) => write!(f, "tr#{}", r),
		&ParamImmediate(v) => write!(f, "{:#x}", v),
		}
	}
}

// vim: ft=rust
