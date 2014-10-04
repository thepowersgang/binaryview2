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
}

// vim: ft=rust
