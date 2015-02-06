// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Block of code in a disassembled program
use std::rc::Rc;
use std::cell::RefCell;
use disasm::state::StateData;
use disasm::instruction::Instruction;
use disasm::{CodePtr,CodeRange};
use std::default::Default;

pub type BlockRef = Rc<RefCell<Block>>;

pub struct Block
{
	instructions: Vec<Instruction>,
	
	refs: Vec<CodePtr>,
	endstate: Option<StateData>,
}

#[derive(Debug,Default)]
pub struct Function
{
	inputs: ::std::collections::BitvSet,
	clobbers: ::std::collections::BitvSet,
}

impl Block
{
	pub fn new(instrs: Vec<Instruction>) -> Block
	{
		debug!("New block for {}", instrs[0].addr());
		Block {
			instructions: instrs,
			refs: Vec::new(),
			endstate: None,
		}
	}
	
	/// Split this block at the specified instruction address
	pub fn split_at(&mut self, addr: CodePtr) -> Block
	{
		let i = match self.instructions.binary_search_by(|e| e.addr().cmp(&addr))
			{
			Ok(i) => i,
			Err(_) => panic!("Address {} not in block ({})", addr, self.range()),
			};
		trace!("i = {}", i);
		let new_instrs = self.instructions.split_off(i);
		
		// Forget state if the block was split
		self.endstate = None;
		Block {
			instructions: new_instrs,
			refs: ::std::mem::replace(&mut self.refs, vec![addr]),
			endstate: None,
		}
	}
	
	pub fn instrs(&self) -> &[Instruction] {
		&self.instructions[]
	}
	pub fn refs(&self) -> &[CodePtr] {
		&self.refs[]
	}
	
	pub fn range(&self) -> ::disasm::CodeRange {
		let first = self.instructions.first().expect("No instructions in block").addr();
		let last  = self.instructions.last(). expect("No instructions in block").addr();
		CodeRange::new(first, last)
	}
	pub fn end_state(&self) -> Option<&StateData> {
		self.endstate.as_ref()
	}
	
	pub fn set_state(&mut self, state: StateData) {
		debug!("State for block {} set to: {:?}", self.range(), state);
		self.endstate = Some(state);
	}
}

impl ::std::cmp::PartialEq<CodePtr> for Block
{
	fn eq(&self, ptr: &CodePtr) -> bool
	{
		self.partial_cmp(ptr).unwrap() == ::std::cmp::Ordering::Equal
	}
}

impl ::std::cmp::PartialOrd<CodePtr> for Block
{
	fn partial_cmp(&self, ptr: &CodePtr) -> Option<::std::cmp::Ordering>
	{
		Some( self.range().contains_ord(*ptr) )
	}
}

// vim: ft=rust
