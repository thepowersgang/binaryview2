//
//
//
use disasm::state::State;
use disasm::InstrParam;

pub trait UCodeOp
{
	fn forwards(&self, instate: &mut State, params: &[InstrParam]);
	fn backwards(&self, instate: &mut State, params: &[InstrParam]);
}

struct UCodeJump;
//struct UCodeCall;

pub static JUMP: UCodeJump = UCodeJump;

impl UCodeOp for UCodeJump
{
	fn forwards(&self, state: &mut State, params: &[InstrParam])
	{
		let target = state.get( params[0] );
		state.add_target( target );
		// TODO: Clear or otherwise munge the state, since jump doesn't continue
	}
	fn backwards(&self, state: &mut State, params: &[InstrParam])
	{
		fail!("Running a jump backwards is impossible");
	}
}

// vim: ft=rust

