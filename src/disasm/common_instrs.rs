//
//
//
use disasm::InstructionClass;
use disasm::{InstrParam};
use disasm::microcode;
use disasm::microcode::UCodeOp;
use disasm::state::State;
use value::Value;

struct IClassJump;
struct IClassCall;
struct IClassMove;
struct IClassLoadOfs;

pub static JUMP: IClassJump = IClassJump;
pub static CALL: IClassCall = IClassCall;
pub static MOVE: IClassMove = IClassMove;
pub static LOAD_OFS: IClassLoadOfs = IClassLoadOfs;

impl InstructionClass for IClassJump
{
	fn name(&self) -> &str {
		"JUMP"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool
	{
		true
	}
	fn print(&self, f: &mut ::std::fmt::Formatter, p: &[InstrParam]) -> Result<(),::std::fmt::FormatError>
	{
		write!(f, "{}", p[0])
	}
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		microcode::JUMP.forwards(state, params.slice(0,1));
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
	}
}

impl InstructionClass for IClassCall
{
	fn name(&self) -> &str {
		"CALL"
	}
	fn is_terminal(&self, _: &[InstrParam]) -> bool
	{
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
	fn is_terminal(&self, _: &[InstrParam]) -> bool
	{
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
		let base = state.get(params[1]);
		let ofs = state.get(params[2]);
		let addr = base + ofs;
		
		let val = state.read(addr);
		state.set(params[0], val);
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		if params[0] != params[1] && params[0] != params[2]
		{
			let addr = state.get(params[1]) + state.get(params[2]);
			let val = state.get(params[0]);
			state.write(addr, val);
		}
		state.set(params[0], Value::unknown());
	}
}

// vim: ft=rust
