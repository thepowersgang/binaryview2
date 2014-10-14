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
	mode: uint,
	addr: u64,
	refs: Vec<BlockRef>,
	//endstate: State<'static>,
}

impl Block
{
	pub fn new_rc(mode: uint, addr: u64) -> BlockRef
	{
		Rc::new( RefCell::new( Block::new(mode, addr) ) )
	}
	
	fn new(mode: uint, addr: u64) -> Block
	{
		debug!("New block for {}:{:#x}", mode, addr);
		Block {
			mode: mode,
			addr: addr,
			refs: Vec::new(),
			//endstate: State::null(),
		}
	}
}

// vim: ft=rust
