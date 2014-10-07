//
//
//
#![macro_escape]
use disasm::InstructionClass;
use disasm::{InstrParam,ParamTmpReg};
use disasm::InstrSizeNA;
use disasm::microcode;
use disasm::microcode::UCodeOp;
use disasm::state::State;
use value::Value;

macro_rules! def_instr{
	($name:ident, $class:ident, ($fmt:ident, $params:ident, $state:ident) => {$isterm:block; $print:block;$forwards:block;$backwards:block;} )
	=> {
	struct $class;
	pub static $name: $class = $class;
	impl ::disasm::InstructionClass for $class
	{
		fn name(&self) -> &str { stringify!($name) }
		fn is_terminal(&self, $params: &[InstrParam]) -> bool { $isterm }
		fn print(&self, $fmt: &mut ::std::fmt::Formatter, $params: &[InstrParam]) -> Result<(),::std::fmt::FormatError> {
			$print
		}
		fn forwards(&self, $state: &mut State, $params: &[InstrParam]) {
			$forwards
		}
		fn backwards(&self, $state: &mut State, $params: &[InstrParam]) {
			$backwards
		}
	}
	}
}

struct IClassCall;
struct IClassMove;
struct IClassShl;
struct IClassAdd;
struct IClassLoadOfs;
struct IClassStoreOfs;

pub static CALL: IClassCall = IClassCall;
pub static MOVE: IClassMove = IClassMove;
pub static SHL: IClassShl = IClassShl;
pub static ADD: IClassAdd = IClassAdd;
pub static LOAD_OFS: IClassLoadOfs = IClassLoadOfs;
pub static STORE_OFS: IClassStoreOfs = IClassStoreOfs;

def_instr!(JUMP, IClassJump, (f,p,state) => {
	{ true };
	{ write!(f, "{}", p[0]) };
	{ microcode::JUMP.forwards(state, InstrSizeNA, p.slice(0,1)); } ;
	{ } ;
})

/*
struct IClassJump;
pub static JUMP: IClassJump = IClassJump;
impl InstructionClass for IClassJump
{
	fn name(&self) -> &str { "JUMP" }
	fn is_terminal(&self, _: &[InstrParam]) -> bool { true }
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}", p[0])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		microcode::JUMP.forwards(state, InstrSizeNA, params.slice(0,1));
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
	}
}
*/

impl InstructionClass for IClassCall
{
	fn name(&self) -> &str {
		"CALL"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool {
		false
	}
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}", p[0])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		//microcode::CALL.forwards(state, params[0..1]);
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
	}
}

impl InstructionClass for IClassMove
{
	fn name(&self) -> &str {
		"MOVE"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool {
		false
	}
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}, {}", p[0], p[1])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		let val = state.get(params[1]);
		state.set(params[0], val);
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		let val = state.get(params[0]);
		state.set(params[0], Value::unknown());
		state.set(params[1], val);
	}
}

impl InstructionClass for IClassShl
{
	fn name(&self) -> &str { "SHL" }
	fn is_terminal(&self, _: &[InstrParam]) -> bool { false }
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		unimplemented!();
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam]) {
		unimplemented!();
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam]) {
		unimplemented!();
	}
}

impl InstructionClass for IClassAdd
{
	fn name(&self) -> &str {
		"ADD"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool {
		false
	}
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}, {}, {}", p[0], p[1], p[2])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		let val = state.get(params[1]) + state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		unimplemented!();
	}
}

impl InstructionClass for IClassLoadOfs
{
	fn name(&self) -> &str {
		"LOAD"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool
	{
		false
	}
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}, [{}+{}]", p[0], p[1], p[2])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		let addr = state.get(params[1]) + state.get(params[2]);
		state.set( ParamTmpReg(0), addr );
		microcode::LOAD.forwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		if params[0] != params[1] && params[0] != params[2]
		{
			let addr = state.get(params[1]) + state.get(params[2]);
			state.set( ParamTmpReg(0), addr );
		}
		microcode::LOAD.backwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	}
}

impl InstructionClass for IClassStoreOfs
{
	fn name(&self) -> &str { "STORE" }
	fn is_terminal(&self, _: &[InstrParam]) -> bool { false }
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "[{}+{}], {}", p[1], p[2], p[0])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		let addr = state.get(params[1]) + state.get(params[2]);
		state.set( ParamTmpReg(0), addr );
		microcode::STORE.forwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		let addr = if params[0] != params[1] && params[0] != params[2] {
				state.get(params[1]) + state.get(params[2])
			}
			else {
				Value::unknown()
			};
		state.set(ParamTmpReg(0), addr);
		microcode::STORE.backwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	}
}

// vim: ft=rust
