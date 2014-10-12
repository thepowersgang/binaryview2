// BinaryView2
// - By John Hodge (thePowetsGang)
//
// disasm/common_instrs.rs
// - Common generic instructions
#![macro_escape]
use disasm::InstructionClass;
use disasm::{InstrParam,ParamTmpReg,ParamImmediate};
use disasm::{InstrSizeNA,InstrSize8,InstrSize16,InstrSize32,InstrSize64};
use disasm::microcode;
use disasm::microcode::UCodeOp;
use disasm::state::State;
use disasm::state::{FlagCarry};
use value::{Value,ValueBool,ValueType};

macro_rules! def_instr{
	($name:ident, $class:ident, ($fmt:ident, $instr:ident, $params:ident, $state:ident) => {$isterm:block; $print:block;$forwards:block;$backwards:block;} )
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
		fn forwards(&self, $state: &mut State, $instr: &::disasm::Instruction) {
			let $params = $instr.params();
			$forwards
		}
		fn backwards(&self, $state: &mut State, $instr: &::disasm::Instruction) {
			let $params = $instr.params();
			let _ = $params;
			let _ = $state;
			$backwards
		}
	}
	}
}

/// Dispatch a call to a function using different size parameters depending on the instruction size
macro_rules! size_call{
	($size:expr, $fcn:ident($($args:expr),+)) => {
		match $size
		{
		InstrSizeNA => fail!(concat!("InstrSizeNA when calling ", stringify!($fcn))),
		InstrSize8  => $fcn::<u8> ($($args),+),
		InstrSize16 => $fcn::<u16>($($args),+),
		InstrSize32 => $fcn::<u32>($($args),+),
		InstrSize64 => $fcn::<u64>($($args),+),
		}
	}
}

// JUMP - Shift program execution elsewhere
def_instr!(JUMP, IClassJump, (f,instr,p,state) => {
	{ true };
	{ write!(f, "{}", p[0]) };
	{
		let target = state.get( p[0] );
		state.add_target( target, instr.mode() );
	};
	{ fail!("Can't reverse a JUMP"); };
})

// CALL - Subroutine call
// TODO: Needs to handle state munging from subroutine clobbers
def_instr!(CALL, IClassCall, (f,instr,p,state) => {
	{ false };
	{ write!(f, "{}", p[0]) };
	{
		microcode::CALL.forwards(state, InstrSizeNA, [p[0], ParamImmediate(instr.mode() as u64)]);
	};
	{
		fail!("TODO: CALL.backwards");
	};
})

// MOVE - Shift a value between registers
def_instr!(MOVE, IClassMove, (f,instr,params,state) => {
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
def_instr!(SHL, IClassShl, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{} := {} << {}", params[0], params[1], params[2]) };
	{
		let v = state.get(params[1]);
		let count = state.get(params[2]);
		if let Some(c) = count.val_known()
		{
			let (ov,cf) = size_call!( instr.opsize(), shl_fwds(v.truncate(), c as uint) );
			state.set(params[0], ov);
			state.flag_set(FlagCarry, cf);
		}
		else
		{
			warn!("TODO: SHL by a set/range of values");
			state.set(params[0], Value::unknown());
		}
	};
	{ unimplemented!(); };
})
fn shl_fwds<T:ValueType>(val: Value<T>, count: uint) -> (Value<u64>,ValueBool)
{
	if count >= val.bitsize() {
		(Value::known(0), ::value::ValueBoolUnknown)
	}
	else {
		let (extra,res) = val << count;
		(res.zero_extend(), extra.bit(0))
	}
}

// SHR - Bitwise Shift Right
def_instr!(SHR, IClassShr, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{} := {} >> {}", params[0], params[1], params[2]) };
	{
		let v = state.get(params[1]);
		let count = state.get(params[2]);
		if let Some(c) = count.val_known()
		{
			let (res, cf) = size_call!( instr.opsize(), shr_fwds(v.truncate(), c as uint) );
			state.set(params[0], res);
			state.flag_set(FlagCarry, cf);
		}
		else
		{
			warn!("TODO: SHL by a set/range of values");
			state.set(params[0], Value::unknown());
			state.flag_set(FlagCarry, ::value::ValueBoolUnknown);
		}
	};
	{ unimplemented!(); };
})
fn shr_fwds<T:ValueType>(val: Value<T>, c: uint) -> (Value<u64>,ValueBool)
{
	if c > val.bitsize() {
		(Value::known(0), ::value::ValueBoolFalse)
	}
	else {
		let (extra,res) = val >> c as uint;
		(res.zero_extend(), extra.bit(0))
	}
}


// ROR - Bitwise Rotate Right
def_instr!(ROR, IClassRor, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{} := {} >>> {}", params[0], params[1], params[2]) };
	{
		let v = state.get(params[1]);
		let count = state.get(params[2]);
		if let Some(c) = count.val_known()
		{
			let (res, ) = size_call!( instr.opsize(), ror_fwds(v.truncate(), c as uint) );
			state.set(params[0], res);
		}
		else
		{
			warn!("TODO: ROR by a set/range of values");
			state.set(params[0], Value::unknown());
		}
	};
	{ unimplemented!(); };
})
fn ror_fwds<T:ValueType>(v: Value<T>, c: uint) -> (Value<u64>,)
{
	if c >= v.bitsize() {
		(Value::known(0), )
	}
	else {
		let (extra,res) = v >> c;
		( (res | extra).zero_extend(), )
	}
}

fn carry_val<T:ValueType>(c: ValueBool) -> Value<T>
{
	match c
	{
	// TODO: Convert unknown into a two value set
	::value::ValueBoolUnknown => Value::unknown(),
	// NOTE: .truncate() can zero extend too
	::value::ValueBoolTrue  => Value::known(1u64).truncate(),
	::value::ValueBoolFalse => Value::known(0u64).truncate(),
	}
}
// ADD - Addition of two values into a register
def_instr!(ADD, IClassAdd, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let a = state.get(params[1]);
		let b = state.get(params[2]);
		let ci = state.flag_get(::disasm::state::FlagCarry);
		let (val, cf) = size_call!( instr.opsize(), add_fwds(a.truncate(), b.truncate(), ci) );
		state.set(params[0], val);
		state.flag_set(::disasm::state::FlagCarry, cf);
	};
	{
		unimplemented!();
	};
})
fn add_fwds<T:ValueType>(a: Value<T>, b: Value<T>, c: ValueBool) -> (Value<u64>, ValueBool)
{
	let carry_in = carry_val(c);
	let rv = a + b + carry_in;
	// TODO: Set flags based on val (requires changing return of "Add<Value>")
	let co = ::value::ValueBoolUnknown;
	(rv.zero_extend(), co)
}

// SUB - Subtraction of two values into a register
def_instr!(SUB, IClassSub, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let a = state.get(params[1]);
		let b = state.get(params[2]);
		let ci = state.flag_get(::disasm::state::FlagCarry);
		let (val, borrowed) = size_call!( instr.opsize(), sub_fwds(a.truncate(), b.truncate(), ci) );
		state.set(params[0], val);
		state.flag_set(::disasm::state::FlagCarry, borrowed);
	};
	{
		unimplemented!();
	};
})
fn sub_fwds<T:ValueType>(a: Value<T>, b: Value<T>, c: ValueBool) -> (Value<u64>, ValueBool)
{
	let borrow = carry_val(c);
	let rv = a - b - borrow;
	// TODO: Support borrow out
	let borrow_out = ::value::ValueBoolUnknown;
	(rv.zero_extend(), borrow_out)
}

// AND - bitwise AND of two values into a register
def_instr!(AND, IClassAnd, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let val = state.get(params[1]) & state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val (e.g. ZF)
	};
	{
		unimplemented!();
	};
})

// Bitwise OR of two values into a register
def_instr!(OR, IClassOr, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let val = state.get(params[1]) | state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val
	};
	{
		unimplemented!();
	};
})

// Bitwise Exclusive OR of two values into a register
def_instr!(XOR, IClassXor, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let val = state.get(params[1]) ^ state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val
	};
	{
		unimplemented!();
	};
})

// MUL - Multiply two values into a register
def_instr!(MUL, IClassMul, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
	{
		let (_hi,val) = state.get(params[1]) * state.get(params[2]);
		state.set(params[0], val);
		// TODO: Set flags based on val
	};
	{
		unimplemented!();
	};
})


def_instr!(NOT, IClassNot, (f, instr, params, state) => {
	{ false };
	{ write!(f, "{}, {}", params[0], params[1]) };
	{
		let val = !state.get(params[1]);
		state.set(params[0], val);
	};
	{
		// Reverse, just read from #0 and write to #1
		let val = !state.get(params[0]);
		state.set(params[1], val);
	};
})

// LOAD (OFS) - Load from a register+offset
def_instr!(LOAD_OFS, IClassLoadOfs, (f, instr, params, state) => {
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
def_instr!(STORE_OFS, IClassStoreOfs, (f, instr, params, state) => {
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
