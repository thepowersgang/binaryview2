// BinaryView: Inteligent Disassembler
// - By John Hodge (thePowersGang)
//
// value.rs
// - A dynamic (unknown, partially known, or known) value
//
// A core type to BinaryView, this represents a register value during execution and a possible value
// for RAM.
use num::{Zero,One,NumCast};
use std::fmt::LowerHex;
use std::cmp::Ordering;

/// Trait for valid values in a value (only implemented for unsigned sized integers)
pub trait ValueType : ::num::PrimInt + LowerHex {
}
impl ValueType for u8 {
}
impl ValueType for u16 {
}
impl ValueType for u32 {
}
impl ValueType for u64 {
}

/// A dynamic value (range determined during execution)
#[derive(Clone)]
pub enum Value<T: ValueType>
{
	/// Value is unknown, but has meaning
	Input(u8),
	/// Value is completely unknown (or at least non-trivial)
	Unknown,
	/// Fully known value
	Known(T),
	// TODO: Support value sets
	//Set(Rc<Vec<T>>),
	// TODO: Support range+mask (or similar)
	//Masked(T,T),	// (Value,KnownFlag)
	// TODO: Support multi-state, e.g. Unknown or a set of possible values
	// - That would be messy to work with, and probably not needed?
	//Nested(Rc<Vec<Value<T>>>),
}

#[derive(PartialEq,Copy,Clone,Debug)]
pub enum ValueBool
{
	True,
	False,
	Unknown,
}

struct ValuePossibilities<'a,T:ValueType+'a>
where
	<T as ::num::traits::Num>::FromStrRadixErr: 'a
{
	val: &'a Value<T>,
	idx: usize,
}

impl<T: ValueType> Value<T>
{
	// ---
	// Type constructors
	// ---
	/// Completely unknown value
	pub fn unknown() -> Value<T> {
		Value::Unknown
	}
	/// Fully known value
	pub fn known(val: T) -> Value<T> {
		Value::Known(val)
	}
	/// Fully known zero (shortcut)
	pub fn zero() -> Value<T> {
		Value::Known( Zero::zero() )
	}
	/// Fully known negative one (shortcut)
	pub fn ones() -> Value<T> {
		Value::Known( Value::<T>::ones_raw() )
	}
	//// A set of possible values
	//pub fn set(vals: Vec<T>) -> Value<T> {
	//	ValueSet(Rc::new(vals))
	//}
	
	fn ones_raw() -> T {
		T::max_value()
	}
	
	fn _bitsize() -> usize {
		::std::mem::size_of::<T>() * 8
	}
	
	// ---
	// Conversions
	// ---
	/// (internal) Cast from one type to another
	fn cast<U: ValueType>(val: U) -> T {
		let mask = if Value::<T>::_bitsize() < Value::<U>::_bitsize() {
				NumCast::from( Value::<T>::ones_raw() ).unwrap()
			} else {
				Value::<U>::ones_raw()
			};
		match NumCast::from(val & mask)
		{
		Some(v) => v,
		None =>
			panic!("Unable to cast {:#x} from u{} to u{}",
				val,
				U::zero().leading_zeros(), //::std::intrinsics::type_name::<U>(),
				T::zero().leading_zeros() //::std::intrinsics::type_name::<T>()
				),
		}
	}
	
	/// Concatenate two values into a larger value
	/// U must be half the size of T
	pub fn concat<U: ValueType>(left: Value<U>, right: Value<U>) -> Value<T>
	{
		assert_eq!( ::std::mem::size_of::<U>() * 2, ::std::mem::size_of::<T>() );
		match (left,right)
		{
		(Value::Known(a),Value::Known(b)) => {
			let a_u: T = NumCast::from(a).unwrap();
			let b_u: T = NumCast::from(b).unwrap();
			Value::Known(a_u | b_u << 8*::std::mem::size_of::<U>())
			}
		_ => Value::Unknown,	// TODO: Handle mask+value (or similar)
		}
	}
	
	/// Return the number of bits in the type
	pub fn bitsize(&self) -> usize {
		::std::mem::size_of::<T>() * 8
	}
	
	/// Truncate (or zero-extend) a value into another size
	pub fn truncate<U: ValueType>(&self) -> Value<U>
	{
		match self
		{
		&Value::Known(a) => {
			let a_u: U = Value::<U>::cast(a);
			Value::Known(a_u)
			}
		&Value::Input(_) => Value::Unknown,
		&Value::Unknown => Value::Unknown,
		}
	}
	pub fn zero_extend<U: ValueType>(&self) -> Value<U> { self.truncate() }
	
	/// Returns Some(val) if the value is fixed
	pub fn val_known(&self) -> Option<T>
	{
		match self
		{
		&Value::Known(v) => Some(v),
		_ => None,
		}
	}
	
	pub fn is_unknown(&self) -> bool
	{
		match self
		{
		&Value::Unknown => true,
		_ => false,
		}
	}
	
	pub fn is_fixed_set(&self) -> bool
	{
		match self
		{
		&Value::Input(_) => false,
		&Value::Unknown => false,
		&Value::Known(_) => true,
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
	
	/// Fetch the value of the specified bit
	pub fn bit(&self, pos: usize) -> ValueBool
	{
		let one: T = One::one();
		let mask = one << pos;
		match self
		{
		&Value::Input(_) => ValueBool::Unknown,
		&Value::Unknown => ValueBool::Unknown,
		&Value::Known(v) =>
			if v & mask != Zero::zero() {
				ValueBool::True
			}
			else {
				ValueBool::False
			},
		}
	}
}

// --------------------------------------------------------------------
// Operations on unknown values
// --------------------------------------------------------------------
/// Add two values
impl<T: ValueType> ::std::ops::Add for Value<T>
{
	type Output = Value<T>;
	fn add(self, other: Value<T>) -> Value<T>
	{
		if let Some(v) = self.val_known() {
			if v == Zero::zero() {
				return other;
			}
		}
		if let Some(v) = other.val_known() {
			if v == Zero::zero() {
				return self;
			}
		}
		match (self, other)
		{
		(Value::Unknown,_) => Value::Unknown,
		(_,Value::Unknown) => Value::Unknown,
		(Value::Input(_),_) => Value::Unknown,
		(_,Value::Input(_)) => Value::Unknown,
		(Value::Known(a),Value::Known(b)) => Value::Known(a+b),
		}
	}
}
/// Subtract two values
impl<T: ValueType> ::std::ops::Sub for Value<T>
{
	type Output = Value<T>;
	fn sub(self, other: Value<T>) -> Value<T>
	{
		if let Some(v) = other.val_known() {
			if v == Zero::zero() {
				// - Subtracting nothing, pass value through unmolested
				return self;
			}
		}
		match (self, other)
		{
		// - Pure unknown poisons
		(Value::Unknown,_) => Value::Unknown,
		(_,Value::Unknown) => Value::Unknown,
		(Value::Input(_),_) => Value::Unknown,
		(_,Value::Input(_)) => Value::Unknown,
		// - Known resolves
		(Value::Known(a),Value::Known(b)) => Value::Known(a-b),
		}
	}
}
/// Multiply two values
/// Returns a pair of values - Upper and lower parts of the result
impl<T: ValueType> ::std::ops::Mul for Value<T>
{
	type Output = (Value<T>, Value<T>);
	fn mul(self, other: Value<T>) -> (Value<T>,Value<T>)
	{
		// Known values - Handle zero and one
		if let Some(v) = other.val_known() {
			if v == Zero::zero() {
				return (Value::zero(), Value::zero());
			}
			if v == One::one() {
				return (Value::zero(), self);
			}
		}
		// Known values - Handle zero and one
		if let Some(v) = self.val_known() {
			if v == Zero::zero() {
				return (Value::zero(), Value::zero());
			}
			if v == One::one() {
				return (Value::zero(), other);
			}
		}
		match (self, other)
		{
		// Otherwise, unknown values are poisonous
		(Value::Unknown,_) => (Value::Unknown,Value::Unknown),
		(_,Value::Unknown) => (Value::Unknown,Value::Unknown),
		(Value::Input(_),_) => (Value::Unknown,Value::Unknown),
		(_,Value::Input(_)) => (Value::Unknown,Value::Unknown),
		// But known values are fixed
		(Value::Known(a),Value::Known(b)) => {
			if a*b < a || a*b < b {
				error!("TODO: Handle overflow in value multiply");
			}
			(Value::zero(),Value::Known(a*b))
			},
		}
	}
}
/// Bitwise AND
impl<T: ValueType> ::std::ops::BitAnd for Value<T>
{
	type Output = Value<T>;
	fn bitand(self, other: Value<T>) -> Value<T>
	{
		// TODO: Restrict range of unknown
		match (self, other)
		{
		// - Zero nukes result
		(_,Value::Known(v)) if v == Zero::zero() => Value::zero(),
		(Value::Known(v),_) if v == Zero::zero() => Value::zero(),
		// - Pure unkown poisons
		(Value::Unknown,_) => Value::Unknown,
		(_,Value::Unknown) => Value::Unknown,
		(Value::Input(_),_) => Value::Unknown,
		(_,Value::Input(_)) => Value::Unknown,
		// - Known resolves
		(Value::Known(a),Value::Known(b)) => Value::Known(a&b),
		}
	}
}
/// Bitwise OR
impl<T: ValueType> ::std::ops::BitOr for Value<T>
{
	type Output = Value<T>;
	fn bitor(self, other: Value<T>) -> Value<T>
	{
		// TODO: Restrict range of unknown
		match (self, other)
		{
		(Value::Unknown,_) => Value::Unknown,
		(_,Value::Unknown) => Value::Unknown,
		(Value::Input(_),_) => Value::Unknown,
		(_,Value::Input(_)) => Value::Unknown,
		(Value::Known(a),Value::Known(b)) => Value::Known(a|b),
		}
	}
}
/// Bitwise Exclusive OR
impl<T: ValueType> ::std::ops::BitXor for Value<T>
{
	type Output = Value<T>;
	fn bitxor(self, other: Value<T>) -> Value<T>
	{
		match (self, other)
		{
		(Value::Unknown,_) => Value::Unknown,
		(_,Value::Unknown) => Value::Unknown,
		(Value::Input(_),_) => Value::Unknown,
		(_,Value::Input(_)) => Value::Unknown,
		(Value::Known(a),Value::Known(b)) => Value::Known(a^b),
		}
	}
}

/// Unary NOT (bitwise)
impl<T: ValueType> ::std::ops::Not for Value<T>
{
	type Output = Value<T>;
	fn not(self) -> Value<T>
	{
		match self
		{
		Value::Input(_) => Value::Unknown,
		Value::Unknown => Value::Unknown,
		Value::Known(a) => Value::Known(!a),
		}
	}
}

/// Logical Shift Left
/// Returns (ShiftedBits, Result)
/// - ShiftedBits are in the lower bits of the value (e.g. -1 << 1 will have the bottom bit set)
impl<T: ValueType> ::std::ops::Shl<usize> for Value<T>
{
	type Output = (Value<T>,Value<T>);
	fn shl(self, rhs: usize) -> (Value<T>,Value<T>)
	{
		if rhs == self.bitsize() {
			(self,Value::zero())
		}
		else if rhs == 0 {
			(Value::zero(),self)
		}
		else {
			match self
			{
			Value::Known(a) => (Value::Known(a>>(self.bitsize()-rhs)), Value::Known(a<<rhs)),
			// TODO: Return a pair of masked values
			_ => (Value::Unknown,Value::Unknown),
			}
		}
	}
}
/// Logical Shift Right
/// Returns (ShiftedBits, Result)
/// - ShiftedBits are in the upper bits of the value (e.g. 1 >> 1 will have the top bit set)
impl<T: ValueType> ::std::ops::Shr<usize> for Value<T>
{
	type Output = (Value<T>, Value<T>);
	fn shr(self, rhs: usize) -> (Value<T>,Value<T>)
	{
		if rhs > self.bitsize() {
			error!("SHR {:?} by {} outside of max shift ({}), clamping", self, rhs, self.bitsize());
			(self,Value::zero())
		}
		else if rhs == self.bitsize() {
			(self,Value::zero())
		}
		else if rhs == 0 {
			(Value::zero(),self)
		}
		else {
			match self
			{
			Value::Known(a) => (Value::Known(a<<(self.bitsize()-rhs)), Value::Known(a>>rhs)),
			// TODO: Return a pair of masked values
			_ => (Value::Unknown,Value::Unknown),
			}
		}
	}
}

//*
impl<T: ValueType> ::std::cmp::PartialEq for Value<T>
{
	fn eq(&self, other: &Value<T>) -> bool
	{
		match ::std::cmp::PartialOrd::partial_cmp(self, other)
		{
		Some(Ordering::Equal) => true,
		_ => false,
		}
	}
}
impl<T: ValueType> ::std::cmp::PartialOrd for Value<T>
{
	fn partial_cmp(&self, other: &Value<T>) -> Option<Ordering>
	{
		match (self,other)
		{
		(&Value::Input(i1), &Value::Input(i2)) => if i1 == i2 { Some(Ordering::Equal) } else { None },
		(&Value::Input(_),_) => None,
		(_,&Value::Input(_)) => None,
		(&Value::Unknown,_) => None,
		(_,&Value::Unknown) => None,
		(&Value::Known(a),&Value::Known(b)) => a.partial_cmp(&b),
		}
	}
}
// */

impl<T: ValueType> ::std::fmt::Debug for Value<T>
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		match self
		{
		&Value::Input(i) => write!(f, "I{}", i),
		&Value::Unknown => write!(f, "?"),
		&Value::Known(v) => write!(f, "{:#x}", v),
		}
	}
}

impl<'a,T: ValueType+'a> Iterator for ValuePossibilities<'a,T>
where
	<T as ::num::traits::Num>::FromStrRadixErr: 'a
{
	type Item = T;
	fn next(&mut self) -> Option<T>
	{
		let rv = match self.val
			{
			&Value::Input(_) => panic!("Can't get possibilities for an unknown value"),
			&Value::Unknown => panic!("Can't get possibilities for an unknown value"),
			&Value::Known(v) => {
				if self.idx == 0 { Some(v) } else { None }
				},
			};
		self.idx += 1;
		rv
	}
}

// vim: ft=rust
