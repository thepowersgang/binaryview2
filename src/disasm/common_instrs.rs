// BinaryView2
// - By John Hodge (thePowetsGang)
//
// disasm/common_instrs.rs
// - Common generic instructions
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
		fn is_terminal(&self, $params: &[InstrParam]) -> bool {
			let _ = $params;
			$isterm
		}
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

// JUMP - Shift program execution elsewhere
def_instr!(JUMP, IClassJump, (f,p,state) => {
	{ true };
	{ write!(f, "{}", p[0]) };
	{ microcode::JUMP.forwards(state, InstrSizeNA, p.slice(0,1)); } ;
	{ fail!("Can't reverse a JUMP"); };
})

// CALL - Subroutine call
// TODO: Needs to handle state munging from subroutine clobbers
def_instr!(CALL, IClassCall, (f,p,state) => {
	{ false };
	{ write!(f, "{}", p[0]) };
	{
		//microcode::CALL.forwards(state, params[0..1]);
	};
	{
		fail!("TODO: CALL.backwards");
	};
})

// MOVE - Shift a value between registers
def_instr!(MOVE, IClassMove, (f,params,state) => {
	{ false };
	{ write!(f, "{}, {}", params[0], params[1]) };
	{
		let val = state.get(params[1]);
		state.set(params[0], val);
	};
	{
		let val = state.get(params[0]);
		state.set(params[0], Value::unknown());
		state.set(params[1], val);
	};
})

// SHL - Bitwise Shift Left
def_instr!(SHL, IClassShl, (f, params, state) => {
	{ false };
	{ write!(f, "{} := {} << {}", params[0], params[1], params[2]) };
	{ unimplemented!(); };
	{ unimplemented!(); };
})

// ADD - Addition of two values into a register
def_instr!(ADD, IClassAdd, (f,params,state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let val = state.get(params[1]) + state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val
	};
	{
		unimplemented!();
	};
})

// LOAD (OFS) - Load from a register+offset
def_instr!(LOAD_OFS, IClassLoadOfs, (f,params,state) => {
	{ false };
	{ write!(f, "{}, [{}+{}]", params[0], params[1], params[2]) };
	{
		let addr = state.get(params[1]) + state.get(params[2]);
		state.set( ParamTmpReg(0), addr );
		microcode::LOAD.forwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	};
	{
		if params[0] != params[1] && params[0] != params[2]
		{
			let addr = state.get(params[1]) + state.get(params[2]);
			state.set( ParamTmpReg(0), addr );
		}
		microcode::LOAD.backwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	};
})

// STORE (OFS) - Store using an offset from a register
def_instr!(STORE_OFS, IClassStoreOfs, (f,params,state) => {
	{ false };
	{ write!(f, "[{}+{}], {}", params[1], params[2], params[0]) };
	{
		let addr = state.get(params[1]) + state.get(params[2]);
		state.set( ParamTmpReg(0), addr );
		microcode::STORE.forwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	};
	{
		let addr = if params[0] != params[1] && params[0] != params[2] {
				state.get(params[1]) + state.get(params[2])
			}
			else {
				Value::unknown()
			};
		state.set(ParamTmpReg(0), addr);
		microcode::STORE.backwards(state, ::disasm::InstrSize32, [params[0], ParamTmpReg(0)]);
	};
})

// vim: ft=rust
