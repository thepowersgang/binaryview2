// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Block of code in a disassembled program
use std::rc::Rc;
use std::cell::RefCell;
use disasm::state::StateData;
use disasm::instruction::Instruction;
use disasm::CodePtr;
use std::default::Default;

pub type BlockRef = Rc<RefCell<Block>>;

pub struct Block
{
	first_ip: ::disasm::CodePtr,
	last_ip: ::disasm::CodePtr,
	instructions: Vec<Instruction>,
	
	refs: Vec<BlockRef>,
	endstate: StateData,
}

impl Block
{
	pub fn new(instrs: Vec<Instruction>) -> Block
	{
		debug!("New block for {}", instrs[0].addr());
		Block {
			first_ip: instrs[0].addr(),
			last_ip:  instrs[instrs.len()-1].addr(),
			instructions: instrs,
			refs: Vec::new(),
			endstate: Default::default(),
		}
	}
	
	/// Split this block at the specified instruction address
	pub fn split_at(&mut self, addr: CodePtr) -> Block
	{
		let i = match self.instructions.binary_search_by(|e| e.addr().cmp(&addr))
			{
			Ok(i) => i,
			Err(_) => panic!("Address {} not in block ({} -- {})", addr, self.first_ip, self.last_ip),
			};
		trace!("i = {}", i);
		let new_instrs = self.instructions.split_off(i);
		
		self.last_ip = self.instructions[self.instructions.len()-1].addr();
		Block {
			first_ip: new_instrs.first().unwrap().addr(),
			last_ip: new_instrs.last().unwrap().addr(),
			instructions: new_instrs,
			refs: ::std::mem::replace(&mut self.refs, Default::default()),
			endstate: ::std::mem::replace(&mut self.endstate, Default::default()),
		}
	}
	
	pub fn instrs(&self) -> &[Instruction] {
		&self.instructions[]
	}
	
	pub fn first_addr(&self) -> ::disasm::CodePtr {
		self.first_ip
	}
	pub fn last_addr(&self) -> ::disasm::CodePtr {
		self.last_ip
	}
	pub fn end_state(&self) -> &StateData {
		&self.endstate
	}
	
	pub fn set_state(&mut self, state: StateData) {
		debug!("State for block {} set to: {:?}", self.first_ip, state);
		self.endstate = state;
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
		use std::cmp::Ordering;
		//trace!("partial_cmp - {}--{} vs {}", self.first_ip, self.last_ip, ptr);
		Some(match self.first_ip.cmp( ptr )
		{
		Ordering::Greater => Ordering::Greater,
		Ordering::Equal => Ordering::Equal,
		Ordering::Less => match self.last_ip.cmp(ptr)
			{
			Ordering::Greater => Ordering::Equal,
			Ordering::Equal => Ordering::Equal,
			Ordering::Less => Ordering::Less,
			}
		})
	}
}

// vim: ft=rust
