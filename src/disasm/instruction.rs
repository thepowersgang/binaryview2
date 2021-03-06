// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/instruction.rs
// - Representation of a single disassembled instruction
use super::CodePtr;

/// Condition code value for an instruction that will always be executed
pub static COND_ALWAYS: u8 = 0xFF;

/// Instruction structure
pub struct Instruction
{
	pub ip: CodePtr,
	pub len: u8,
	
	condition: u8,
	opsize: InstrSize,
	pub class: &'static InstructionClass,
	params: Vec<InstrParam>,
	
	is_target: bool,
	is_call_target: bool,
}

/// Instruction parameter
#[derive(PartialEq,Copy,Clone)]
pub enum InstrParam
{
	TrueReg(u8),
	TmpReg(u8),
	Immediate(u64),
}
/// Instruction size
#[derive(PartialEq,Copy,Clone)]
pub enum InstrSize
{
	SizeNA,
	Size8,
	Size16,
	Size32,
	Size64,
}

/// Instruction class trait
pub trait InstructionClass: 'static
{
	fn name(&self) -> &str;
	fn is_terminal(&self, &[InstrParam]) -> bool;
	fn print(&self, &mut ::std::fmt::Formatter, &[InstrParam]) -> ::std::fmt::Result;
	fn forwards(&self, &mut ::disasm::state::State, &Instruction);
	fn backwards(&self, &mut ::disasm::state::State, &Instruction);
}

// --------------------------------------------------------------------
impl Instruction
{
	pub fn invalid() -> Instruction
	{
		Instruction::new(0, COND_ALWAYS, InstrSize::SizeNA, &INVALID, vec![])
	}
	pub fn new(
		len: u8,
		condition: u8,
		opsize: InstrSize,
		class: &'static InstructionClass,
		params: Vec<InstrParam>
		) -> Instruction
	{
		Instruction {
			ip: CodePtr::new(0, 0),
			len: len,
			condition: condition,
			opsize: opsize,
			class: class,
			params: params,
			is_target: false,
			is_call_target: false,
		}
	}
	pub fn set_addr(&mut self, addr: CodePtr) {
		self.ip = addr;
	}
	pub fn set_target(&mut self) {
		self.is_target = true;
	}
	pub fn set_call_target(&mut self) {
		self.is_call_target = true;
	}
	
	pub fn is_target(&self) -> bool { self.is_target }
	pub fn is_call_target(&self) -> bool { self.is_call_target }
	
	pub fn contains(&self, addr: u64) -> bool {
		self.ip.addr() <= addr && addr < self.ip.addr() + self.len as u64
	}
	pub fn is_terminal(&self) -> bool {
		self.condition == COND_ALWAYS && self.class.is_terminal(&self.params)
	}
	pub fn is_conditional(&self) -> bool {
		self.condition != COND_ALWAYS
	}

	pub fn addr(&self) -> CodePtr { self.ip }
	pub fn mode(&self) -> super::CPUMode { self.ip.mode() }
	pub fn opsize(&self) -> InstrSize { self.opsize }
	pub fn params(&self) -> &[InstrParam] { &self.params }
}

impl ::std::fmt::Debug for Instruction
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		try!( write!(f, "[{:?}]+{} ", self.ip, self.len) );
		try!( write!(f, "{{{:?}}}:{:x} {} ", self.opsize, self.condition, self.class.name()) );
		try!( self.class.print(f, &self.params) );
		Ok( () )
	}
}

impl ::std::fmt::Display for Instruction
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		try!( write!(f, "[{}+{}] ", self.ip, self.len) );
		try!( write!(f, "{{{:?}}}:{:x} {} ", self.opsize, self.condition, self.class.name()) );
		try!( self.class.print(f, &self.params) );
		Ok( () )
	}
}

// --------------------------------------------------------------------
impl InstrParam
{
	pub fn immediate(&self) -> u64
	{
		match self
		{
		&InstrParam::Immediate(v) => v,
		_ => panic!("Expected immediate value, got {:?}", self),
		}
	}
}
impl ::std::fmt::Debug for InstrParam
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		match self
		{
		&InstrParam::TrueReg(r) => write!(f, "R{}", r),
		&InstrParam::TmpReg(r) => write!(f, "tr#{}", r),
		&InstrParam::Immediate(v) => write!(f, "{:#x}", v),
		}
	}
}

// --------------------------------------------------------------------
impl ::std::fmt::Debug for InstrSize
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		match self
		{
		&InstrSize::SizeNA => write!(f, "NA"),
		&InstrSize::Size8  => write!(f, " 8"),
		&InstrSize::Size16 => write!(f, "16"),
		&InstrSize::Size32 => write!(f, "32"),
		&InstrSize::Size64 => write!(f, "64"),
		}
	}
}

def_instr!{INVALID, IClassInvalid, (f,i,_p,_s) => {
	{ true };
	{ write!(f, "--") };
	{
	};
	{
	};
}}

// vim: ft=rust
