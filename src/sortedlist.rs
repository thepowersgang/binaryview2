//
//
//
use std::slice::{Found,NotFound};

pub trait SortedList<T>
{
	fn find_ins<'a>(&'a mut self, sort: |&T|->Ordering) -> VecInsertPos<'a,T>;
}

pub struct VecInsertPos<'a,T:'a>
{
	vec: &'a mut Vec<T>,
	pos: uint,
}

impl<T> SortedList<T> for Vec<T>
{
	fn find_ins<'s>(&'s mut self, order: |&T|->Ordering) -> VecInsertPos<'s,T>
	{
		let pos = match self.as_mut_slice().binary_search(order)
			{
			Found(a) => a,
			NotFound(a) => a,
			};
		VecInsertPos {
			vec: self,
			pos: pos,
		}
	}
}

impl<'a,T> VecInsertPos<'a,T>
{	
	pub fn is_end(&self) -> bool {
		self.pos == self.vec.len()
	}
	pub fn next<'b>(&'b self) -> &'b T {
		assert!( !self.is_end() );
		&(*self.vec)[self.pos]
	}
	pub fn insert(&mut self, val: T) {
		self.vec.insert(self.pos, val);
		self.pos += 1;
	}
}

// vim: ft=rust
