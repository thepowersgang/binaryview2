// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/instruction.rs
// - Representation of a single disassembled instruction

/// Condition code value for an instruction that will always be executed
pub static COND_ALWAYS: u8 = 0xFF;

/// Instruction structure
pub struct Instruction
{
	pub mode: uint,
	pub base: u64,
	pub len: u8,
	
	condition: u8,
	opsize: InstrSize,
	pub class: &'static InstructionClass + 'static,
	params: Vec<InstrParam>,
	
	is_target: bool,
	is_call_target: bool,
}

/// Instruction parameter
#[deriving(PartialEq)]
pub enum InstrParam
{
	ParamTrueReg(u8),
	ParamTmpReg(u8),
	ParamImmediate(u64),
}
/// Instruction size
pub enum InstrSize
{
	InstrSizeNA,
	InstrSize8,
	InstrSize16,
	InstrSize32,
	InstrSize64,
}

/// Instruction class trait
pub trait InstructionClass
{
	fn name(&self) -> &str;
	fn is_terminal(&self, &[InstrParam]) -> bool;
	fn print(&self, &mut ::std::fmt::Formatter, &[InstrParam]) -> Result<(),::std::fmt::FormatError>;
	fn forwards(&self, &mut ::disasm::state::State, &Instruction);
	fn backwards(&self, &mut ::disasm::state::State, &Instruction);
}

// --------------------------------------------------------------------
impl Instruction
{
	pub fn invalid() -> Instruction
	{
		Instruction::new(0, COND_ALWAYS, InstrSizeNA, &INVALID, vec![])
	}
	pub fn new(
		len: u8,
		condition: u8,
		opsize: InstrSize,
		class: &'static InstructionClass + 'static,
		params: Vec<InstrParam>
		) -> Instruction
	{
		Instruction {
			mode: 0,
			base: 0,
			len: len,
			condition: condition,
			opsize: opsize,
			class: class,
			params: params,
			is_target: false,
			is_call_target: false,
		}
	}
	pub fn set_addr(&mut self, addr: u64, mode: uint) {
		self.mode = mode;
		self.base = addr;
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
		self.base <= addr && addr < self.base + self.len as u64
	}
	pub fn is_terminal(&self) -> bool {
		self.condition == COND_ALWAYS && self.class.is_terminal(self.params.as_slice())
	}

	pub fn mode(&self) -> uint { self.mode }
	pub fn opsize(&self) -> InstrSize { self.opsize }
	pub fn params(&self) -> &[InstrParam] { self.params.as_slice() }
}

impl ::std::fmt::Show for Instruction
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(),::std::fmt::FormatError>
	{
		try!( write!(f, "[{}:{:8x}]+{:u} ", self.mode, self.base, self.len) );
		try!( write!(f, "{{{}}}:{:x} {} ", self.opsize, self.condition, self.class.name()) );
		try!( self.class.print(f, self.params.as_slice()) );
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
		&ParamImmediate(v) => v,
		_ => fail!("Expected immediate value, got {}", self),
		}
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

// --------------------------------------------------------------------
impl ::std::fmt::Show for InstrSize
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(),::std::fmt::FormatError>
	{
		match self
		{
		&InstrSizeNA => write!(f, "NA"),
		&InstrSize8  => write!(f, " 8"),
		&InstrSize16 => write!(f, "16"),
		&InstrSize32 => write!(f, "32"),
		&InstrSize64 => write!(f, "64"),
		}
	}
}

def_instr!(INVALID, IClassInvalid, (f,i,p,s) => {
	{ true };
	{ write!(f, "--") };
	{
		let _ = p;
		let _ = s;
	};
	{
		let _ = p;
		let _ = s;
	};
})

// vim: ft=rust
