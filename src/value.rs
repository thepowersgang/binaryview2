// BinaryView: Inteligent Disassembler
// - By John Hodge (thePowersGang)
//
// value.rs
// - A dynamic (unknown, partially known, or known) value
//
// A core type to BinaryView, this represents a register value during execution and a possible value
// for RAM.

/// A dynamic value (range determined during execution)
#[deriving(Clone)]
pub enum Value<T: Int>
{
	ValueUnknown,
	ValueKnown(T),
	// TODO: Support value sets
	// TODO: Support range+mask (or similar)
	// TODO: Support multi-state, e.g. Unknown or a set of possible values
}

struct ValuePossibilities<'a,T:Int+'static>
{
	val: &'a Value<T>,
	idx: uint,
}

impl<T: Int> Value<T>
{
	pub fn unknown() -> Value<T>
	{
		ValueUnknown
	}
	pub fn fixed(val: T) -> Value<T>
	{
		ValueKnown(val)
	}
	pub fn zero_extend<U: Unsigned+Int>(val: Value<U>) -> Value<T>
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
	pub fn concat<U: Int>(left: Value<U>, right: Value<U>) -> Value<T>
	{
		match (left,right)
		{
		(ValueKnown(a),ValueKnown(b)) => {
			let a_u: T = NumCast::from(a).unwrap();
			let b_u: T = NumCast::from(b).unwrap();
			ValueKnown(a_u | b_u << (4*::std::mem::size_of::<T>()))
			}
		_ => ValueUnknown,
		}
	}
	
	pub fn truncate<U: Int+Unsigned>(&self) -> Value<U>
	{
		match self
		{
		&ValueKnown(a) => {
			let a_u: U = NumCast::from(a).unwrap();
			ValueKnown(a_u)
			}
		&ValueUnknown => ValueUnknown,
		}
	}
	
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
	
	pub fn possibilities<'s>(&'s self) -> ValuePossibilities<'s,T>
	{
		ValuePossibilities {
			val: self,
			idx: 0,
		}
	}
}

impl<T: Int> ::std::ops::Add<Value<T>,Value<T>> for Value<T>
{
	fn add(&self, other: &Value<T>) -> Value<T>
	{
		match (self, other)
		{
		(&ValueUnknown,_) => ValueUnknown,
		(_,&ValueUnknown) => ValueUnknown,
		(&ValueKnown(a),&ValueKnown(b)) => ValueKnown(a+b),
		}
	}
}

impl<T: Int+::std::fmt::LowerHex> ::std::fmt::Show for Value<T>
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

impl<'a,T: Int> Iterator<T> for ValuePossibilities<'a,T>
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
