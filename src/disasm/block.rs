// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Block of code in a disassembled program
use std::rc::Rc;
use std::cell::RefCell;
use disasm::state::StateData;

pub type BlockRef = Rc<RefCell<Block>>;

pub struct Block
{
	ip: ::disasm::CodePtr,
	last_ip: ::disasm::CodePtr,
	refs: Vec<BlockRef>,
	endstate: StateData,
}

impl Block
{
	pub fn new_rc(ip: ::disasm::CodePtr) -> BlockRef
	{
		Rc::new( RefCell::new( Block::new(ip) ) )
	}
	
	fn new(ip: ::disasm::CodePtr) -> Block
	{
		debug!("New block for {}", ip);
		Block {
			ip: ip,
			last_ip: ip,
			refs: Vec::new(),
			endstate: ::std::default::Default::default(),
		}
	}
	
	pub fn last_instr(&self) -> ::disasm::CodePtr {
		self.last_ip
	}
	pub fn end_state(&self) -> &StateData {
		&self.endstate
	}
	
	pub fn set_last_instr(&mut self, addr: ::disasm::CodePtr) {
		self.last_ip = addr;
	}
	pub fn set_state(&mut self, state: StateData) {
		debug!("State for block {} set to: {:?}", self.ip, state);
		self.endstate = state;
	}
}

// vim: ft=rust
