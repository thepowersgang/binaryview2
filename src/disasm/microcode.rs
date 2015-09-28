//
//
//
use disasm::state::State;
use disasm::instruction::{InstrParam,InstrSize};
use value::Value;

pub trait UCodeOp
{
	fn forwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam]);
	fn backwards(&self, state: &mut State, size: InstrSize, params: &[InstrParam]);
}

macro_rules! def_ucode{
	($name:ident, $class:ident, ($st:ident, $sz:ident, $p:ident) => {$fwd:block; $back:block;})
	=> {
		pub struct $class;
		pub static $name: $class = $class;
		impl UCodeOp for $class
		{
			fn forwards(&self, $st: &mut State, $sz: InstrSize, $p: &[InstrParam])  $fwd
			fn backwards(&self, $st: &mut State, $sz: InstrSize, $p: &[InstrParam]) $back
		}
	};
}

def_ucode!{LOAD, UCodeLoad, (state, size, params) => {
	{
		let addr = state.get(params[1]);
		// Handle zero-extending the value to 64 bits
		let val = match size
			{
			InstrSize::SizeNA => Value::unknown(),
			InstrSize::Size8  => state.read::<u8> (&addr).zero_extend(),
			InstrSize::Size16 => state.read::<u16>(&addr).zero_extend(),
			InstrSize::Size32 => state.read::<u32>(&addr).zero_extend(),
			InstrSize::Size64 => state.read::<u64>(&addr),	// 64 = native
			};
		state.set(params[0], val);
	};
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = state.get(params[0]);
			match size
			{
			InstrSize::SizeNA => {},
			InstrSize::Size8  => state.write(&addr, val.truncate::<u8> ()),
			InstrSize::Size16 => state.write(&addr, val.truncate::<u16>()),
			InstrSize::Size32 => state.write(&addr, val.truncate::<u32>()),
			InstrSize::Size64 => state.write(&addr, val.truncate::<u64>()),
			}
		}
		state.set(params[0], Value::unknown());
	};
}}

def_ucode!{STORE, UCodeStore, (state, size, params) => {
	{
		let addr = state.get(params[1]);
		let val = state.get(params[0]);
		// Handle zero-extending the value to 64 bits
		match size
		{
		InstrSize::SizeNA => {},
		InstrSize::Size8  => state.write(&addr, val.truncate::<u8> ()),
		InstrSize::Size16 => state.write(&addr, val.truncate::<u16>()),
		InstrSize::Size32 => state.write(&addr, val.truncate::<u32>()),
		InstrSize::Size64 => state.write(&addr, val.truncate::<u64>()),
		}
	};
	{
		if params[0] != params[1]
		{
			let addr = state.get(params[1]);
			let val = match size
				{
				InstrSize::SizeNA => Value::unknown(),
				InstrSize::Size8  => state.read::<u8> (&addr).zero_extend(),
				InstrSize::Size16 => state.read::<u16>(&addr).zero_extend(),
				InstrSize::Size32 => state.read::<u32>(&addr).zero_extend(),
				InstrSize::Size64 => state.read::<u64>(&addr).zero_extend(),
				};
			state.set(params[0], val);
		}
		else
		{
			//state.set(params[0], Value::unknown());
		}
	};
}}

// vim: ft=rust

