// BinaryView: Inteligent Disassembler
// - By John Hodge (thePowersGang)
//
// value.rs
// - A dynamic (unknown, partially known, or known) value
//
// A core type to BinaryView, this represents a register value during execution and a possible value
// for RAM.
use std::num::Zero;
use std::fmt::LowerHex;

pub trait ValueType : Int + Unsigned + Zero + LowerHex { }
impl ValueType for u8 {}
impl ValueType for u16 {}
impl ValueType for u32 {}
impl ValueType for u64 {}

/// A dynamic value (range determined during execution)
#[deriving(Clone,PartialEq)]
pub enum Value<T: ValueType>
{
	ValueUnknown,
	ValueKnown(T),
	// TODO: Support value sets
	// TODO: Support range+mask (or similar)
	// TODO: Support multi-state, e.g. Unknown or a set of possible values
}

pub enum ValueBool
{
	ValueBoolTrue,
	ValueBoolFalse,
	ValueBoolUnknown,
}

struct ValuePossibilities<'a,T:ValueType+'static>
{
	val: &'a Value<T>,
	idx: uint,
}

impl<T: ValueType> Value<T>
{
	pub fn unknown() -> Value<T> {
		ValueUnknown
	}
	pub fn known(val: T) -> Value<T> {
		ValueKnown(val)
	}
	pub fn zero() -> Value<T> {
		ValueKnown( Zero::zero() )
	}
	pub fn ones() -> Value<T> {
		let bs = ::std::mem::size_of::<T>() * 8;
		let top: T = NumCast::from( 1u64 << (bs-1) ).unwrap();
		let v = top | (!top);
		ValueKnown( v )
	}
	pub fn cast<U: ValueType>(val: U) -> T {
		match NumCast::from(val)
		{
		Some(v) => v,
		None => unsafe {
			fail!("Unable to cast {:#x} from {} to {}",
				val,
				(*::std::intrinsics::get_tydesc::<U>()).name,
				(*::std::intrinsics::get_tydesc::<T>()).name
				);
			},
		}
	}
	
	/// Zero-extend a value to this type
	pub fn zero_extend<U: ValueType>(val: Value<U>) -> Value<T>
	{
		match val
		{
		ValueKnown(v) => {
			let v_u: T = NumCast::from(v).unwrap();
			ValueKnown(v_u)
			},
		ValueUnknown => ValueUnknown,
		}
	}
	/// Concatenate two values into a larger value
	/// U must be half the size of T
	pub fn concat<U: ValueType>(left: Value<U>, right: Value<U>) -> Value<T>
	{
		assert_eq!( ::std::mem::size_of::<U>() * 2, ::std::mem::size_of::<T>() );
		match (left,right)
		{
		(ValueKnown(a),ValueKnown(b)) => {
			let a_u: T = NumCast::from(a).unwrap();
			let b_u: T = NumCast::from(b).unwrap();
			ValueKnown(a_u | b_u << 8*::std::mem::size_of::<U>())
			}
		_ => ValueUnknown,	// TODO: Handle mask+value (or similar)
		}
	}

	pub fn bitsize(&self) -> uint {
		::std::mem::size_of::<T>() * 8
	}
	
	/// Truncate (or zero-extend) a value into another size
	pub fn truncate<U: ValueType>(&self) -> Value<U>
	{
		match self
		{
		&ValueKnown(a) => {
			let a_u: U = Value::<U>::cast(a);
			ValueKnown(a_u)
			}
		&ValueUnknown => ValueUnknown,
		}
	}
	
	/// Returns Some(val) if the value is fixed
	pub fn val_known(&self) -> Option<T>
	{
		match self
		{
		&ValueKnown(v) => Some(v),
		_ => None,
		}
	}
	
	pub fn is_fixed_set(&self) -> bool
	{
		match self
		{
		&ValueUnknown => false,
		&ValueKnown(_) => true,
		}
	}
	
	/// Get an iterator of possible concrete values for this value
	pub fn possibilities<'s>(&'s self) -> ValuePossibilities<'s,T>
	{
		ValuePossibilities {
			val: self,
			idx: 0,
		}
	}
	
	pub fn bit(&self, pos: uint) -> ValueBool
	{
		let one: T = NumCast::from(1u).unwrap();
		let mask = one << self.bitsize()-1;
		match self
		{
		&ValueUnknown => ValueBoolUnknown,
		&ValueKnown(v) =>
			if v & mask != Zero::zero() {
				ValueBoolTrue
			}
			else {
				ValueBoolFalse
			},
		}
	}
}

// --------------------------------------------------------------------
// Operations on unknown values
// --------------------------------------------------------------------
/// Add two values
impl<T: ValueType> ::std::ops::Add<Value<T>,Value<T>> for Value<T>
{
	fn add(&self, other: &Value<T>) -> Value<T>
	{
		match (self, other)
		{
		(_,&ValueKnown(v)) if v == Zero::zero() => *self,
		(&ValueKnown(v),_) if v == Zero::zero() => *other,
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a+b),
		}
	}
}
/// Subtract two values
impl<T: ValueType> ::std::ops::Sub<Value<T>,Value<T>> for Value<T>
{
	fn sub(&self, other: &Value<T>) -> Value<T>
	{
		match (self, other)
		{
		// - Subtracting nothing, pass value through unmolested
		(_,&ValueKnown(v)) if v == Zero::zero() => *self,
		// - Pure unknown poisons
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		// - Known resolves
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a-b),
		}
	}
}
/// Multiply two values
/// Returns a pair of values - Upper and lower parts of the result
impl<T: ValueType> ::std::ops::Mul<Value<T>,(Value<T>,Value<T>)> for Value<T>
{
	fn mul(&self, other: &Value<T>) -> (Value<T>,Value<T>)
	{
		match (self, other)
		{
		// Either being zero causes the result to be zero
		(_,&ValueKnown(v)) if v == Zero::zero() => (Value::zero(),Value::zero()),
		(&ValueKnown(v),_) if v == Zero::zero() => (Value::zero(),Value::zero()),
		// Otherwise, unknown values are poisonous
		(&ValueUnknown,_) => (ValueUnknown,ValueUnknown),
		(_,&ValueUnknown) => (ValueUnknown,ValueUnknown),
		// But known values are fixed
		(&ValueKnown(a),&ValueKnown(b)) => {
			if a*b < a || a*b < b {
				error!("TODO: Handle overflow in value multiply");
			}
			(Value::zero(),ValueKnown(a*b))
			},
		}
	}
}
/// Bitwise AND
impl<T: ValueType> ::std::ops::BitAnd<Value<T>,Value<T>> for Value<T>
{
	fn bitand(&self, other: &Value<T>) -> Value<T>
	{
		// TODO: Restrict range of unknown
		match (self, other)
		{
		// - Zero nukes result
		(_,&ValueKnown(v)) if v == Zero::zero() => Value::zero(),
		(&ValueKnown(v),_) if v == Zero::zero() => Value::zero(),
		// - Pure unkown poisons
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		// - Known resolves
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a&b),
		}
	}
}
/// Bitwise OR
impl<T: ValueType> ::std::ops::BitOr<Value<T>,Value<T>> for Value<T>
{
	fn bitor(&self, other: &Value<T>) -> Value<T>
	{
		// TODO: Restrict range of unknown
		match (self, other)
		{
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a|b),
		}
	}
}
/// Bitwise Exclusive OR
impl<T: ValueType> ::std::ops::BitXor<Value<T>,Value<T>> for Value<T>
{
	fn bitxor(&self, other: &Value<T>) -> Value<T>
	{
		match (self, other)
		{
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a^b),
		}
	}
}

/// Unary NOT (bitwise)
impl<T: ValueType> ::std::ops::Not<Value<T>> for Value<T>
{
	fn not(&self) -> Value<T>
	{
		match self
		{
		&ValueUnknown => ValueUnknown,
		&ValueKnown(a) => ValueKnown(!a),
		}
	}
}

/// Logical Shift Left
/// Returns (ShiftedBits, Result)
/// - ShiftedBits are in the lower bits of the value (e.g. -1 << 1 will have the bottom bit set)
impl<T: ValueType> ::std::ops::Shl<uint,(Value<T>,Value<T>)> for Value<T>
{
	fn shl(&self, &rhs: &uint) -> (Value<T>,Value<T>)
	{
		if rhs == self.bitsize() {
			(*self,Value::zero())
		}
		else if rhs == 0 {
			(Value::zero(),*self)
		}
		else {
			match self
			{
			&ValueKnown(a) => (ValueKnown(a>>(self.bitsize()-rhs)), ValueKnown(a<<rhs)),
			// TODO: Return a pair of masked values
			_ => (ValueUnknown,ValueUnknown),
			}
		}
	}
}
/// Logical Shift Right
/// Returns (ShiftedBits, Result)
/// - ShiftedBits are in the upper bits of the value (e.g. 1 >> 1 will have the top bit set)
impl<T: ValueType> ::std::ops::Shr<uint,(Value<T>,Value<T>)> for Value<T>
{
	fn shr(&self, &rhs: &uint) -> (Value<T>,Value<T>)
	{
		if rhs > self.bitsize() {
			error!("SHR {} by {} outside of max shift ({}), clamping", self, rhs, self.bitsize());
			(*self,Value::zero())
		}
		else if rhs == self.bitsize() {
			(*self,Value::zero())
		}
		else if rhs == 0 {
			(Value::zero(),*self)
		}
		else {
			match self
			{
			&ValueKnown(a) => (ValueKnown(a<<(self.bitsize()-rhs)), ValueKnown(a>>rhs)),
			// TODO: Return a pair of masked values
			_ => (ValueUnknown,ValueUnknown),
			}
		}
	}
}

impl<T: ValueType> ::std::cmp::PartialOrd for Value<T>
{
	fn partial_cmp(&self, other: &Value<T>) -> Option<Ordering>
	{
		match (self,other)
		{
		(&ValueUnknown,_) => None,
		(_,&ValueUnknown) => None,
		(&ValueKnown(a),&ValueKnown(b)) => a.partial_cmp(&b),
		}
	}
}
//impl<T: ValueType> ::std::cmp::PartialEq for Value<T>
//{
//	fn eq(&self, other: &Value<T>) -> Value<T>
//	{
//		match (self,other)
//		{
//		}
//	}
//}

impl<T: ValueType> ::std::fmt::Show for Value<T>
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(),::std::fmt::FormatError>
	{
		match self
		{
		&ValueUnknown => write!(f, "?"),
		&ValueKnown(v) => write!(f, "{:#x}", v),
		}
	}
}

impl<'a,T: ValueType> Iterator<T> for ValuePossibilities<'a,T>
{
	fn next(&mut self) -> Option<T>
	{
		let rv = match self.val
			{
			&ValueUnknown => fail!("Can't get possibilities for an unknown value"),
			&ValueKnown(v) => {
				if self.idx == 0 { Some(v) } else { None }
				},
			};
		self.idx += 1;
		rv
	}
}

// vim: ft=rust
