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

macro_rules! def_ucode{
	($name:ident, $class:ident, ($st:ident, $sz:ident, $p:ident) => {$fwd:block; $back:block;})
	=> {
		struct $class;
		pub static $name: $class = $class;
		impl UCodeOp for $class
		{
			fn forwards(&self, $st: &mut State, $sz: InstrSize, $p: &[InstrParam])  $fwd
			fn backwards(&self, $st: &mut State, $sz: InstrSize, $p: &[InstrParam]) $back
		}
	};
}

def_ucode!(JUMP, UCodeJump, (state, size, params) => {
	{
		let target = state.get( params[0] );
		state.add_target( target, 0 );	// TODO: Get mode from state
	};
	{
		fail!("Running a jump backwards is impossible");
	};
})

def_ucode!(CALL, UCodeCall, (state, size, params) => {
	{ unimplemented!(); };
	{ unimplemented!(); };
})

def_ucode!(LOAD, UCodeLoad, (state, size, params) => {
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
	};
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = state.get(params[0]);
			state.write(addr, val);
		}
		state.set(params[0], Value::unknown());
	};
})

def_ucode!(STORE, UCodeStore, (state, size, params) => {
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
	};
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = state.get(params[0]);
			state.write(addr, val);
		}
		state.set(params[0], Value::unknown());
	};
})

// Push - Pretty darn simple due to rust
def_ucode!(PUSH, UCodePush, (state, size, params) => {
	{
		let val = state.get(params[0]);
		// TODO: Should this code handle the stack pointer manipulation?
		// - Nah, leave that up to the user
		state.stack_push( val );
	};
	{
		let val = state.stack_pop();
		state.set(params[0], val);
		unimplemented!();
	};
})

// vim: ft=rust

