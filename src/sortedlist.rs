//
//
//
use std::cmp::Ordering;

pub trait SortedList<T>
{
	fn find_ins<'a, F: FnMut(&T)->Ordering>(&'a mut self, sort: F) -> VecInsertPos<'a,T>;
}

pub struct VecInsertPos<'a,T:'a>
{
	vec: &'a mut Vec<T>,
	pos: usize,
}

impl<T> SortedList<T> for Vec<T>
{
	fn find_ins<'s, F: FnMut(&T)->Ordering>(&'s mut self, order: F) -> VecInsertPos<'s,T>
	{
		let pos = match self.binary_search_by(order)
			{
			Ok(a) => a,
			Err(a) => a,
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
