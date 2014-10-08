//
//
//
use disasm::state::State;
use disasm::{InstrParam,InstrSize};
use value::Value;

pub trait UCodeOp
{
	fn forwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam]);
	fn backwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam]);
}

struct UCodeJump;
//struct UCodeCall;
struct UCodeLoad;
struct UCodeStore;

pub static JUMP: UCodeJump = UCodeJump;
pub static LOAD: UCodeLoad = UCodeLoad;
pub static STORE: UCodeStore = UCodeStore;

impl UCodeOp for UCodeJump
{
	fn forwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		let target = state.get( params[0] );
		state.add_target( target, 0 );	// TODO: Get mode from state
		// TODO: Clear or otherwise munge the state, since jump doesn't continue
	}
	fn backwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		fail!("Running a jump backwards is impossible");
	}
}

impl UCodeOp for UCodeLoad
{
	fn forwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		let addr = state.get(params[1]);
		// Handle zero-extending the value to 64 bits
		let val = match size
			{
			::disasm::InstrSizeNA => Value::unknown(),
			::disasm::InstrSize8  => Value::zero_extend::<u8> ( state.read(addr) ),
			::disasm::InstrSize16 => Value::zero_extend::<u16>( state.read(addr) ),
			::disasm::InstrSize32 => Value::zero_extend::<u32>( state.read(addr) ),
			::disasm::InstrSize64 => Value::concat::<u32>(
				state.read(addr),
				state.read(addr+Value::known(4))
				),
			};
		state.set(params[0], val);
	}
	fn backwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = state.get(params[0]);
			state.write(addr, val);
		}
		state.set(params[0], Value::unknown());
	}
}

impl UCodeOp for UCodeStore
{
	fn forwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		let addr = state.get(params[1]);
		let val = state.get(params[0]);
		// Handle zero-extending the value to 64 bits
		match size
		{
		::disasm::InstrSizeNA => {},
		::disasm::InstrSize8  => state.write(addr, val.truncate::<u8> ()),
		::disasm::InstrSize16 => state.write(addr, val.truncate::<u16>()),
		::disasm::InstrSize32 => state.write(addr, val.truncate::<u32>()),
		::disasm::InstrSize64 => state.write(addr, val.truncate::<u64>()),
		}
	}
	fn backwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam])
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = state.get(params[0]);
			state.write(addr, val);
		}
		state.set(params[0], Value::unknown());
	}
}

// vim: ft=rust

