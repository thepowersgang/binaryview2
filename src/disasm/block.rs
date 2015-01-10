// BinaryView2
// - By John Hodge (thePowersGang)
//
// disasm/mod.rs
// - Block of code in a disassembled program
use std::rc::Rc;
use std::cell::RefCell;
use disasm::state::State;

pub type BlockRef = Rc<RefCell<Block>>;

pub struct Block
{
	ip: ::disasm::CodePtr,
	refs: Vec<BlockRef>,
	//endstate: State<'static>,
}

impl Block
{
	pub fn new_rc(ip: ::disasm::CodePtr) -> BlockRef
	{
		Rc::new( RefCell::new( Block::new(ip) ) )
	}
	
	fn new(ip: ::disasm::CodePtr) -> Block
	{
		debug!("New block for {}:{:#x}", ip.1, ip.0);
		Block {
			ip: ip,
			refs: Vec::new(),
			//endstate: State::null(),
		}
	}
}

// vim: ft=rust
