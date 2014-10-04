//
//
//
#![feature(associated_types)]
#![feature(phase)]

#[phase(plugin,link)] extern crate log;
extern crate getopts;

mod sortedlist;	// Trait - Provides a sorted list interface to generic types

mod value;	// Value type
mod memory;	// Memory
mod types;	// Type manager
mod disasm;	// Disassembler
//mod analyse;	// Analysis of the disassembled code (to produce more addresses, and get functions)

static MAX_LOOPS: uint = 16;	// Maximum number of passes during disassembly+processing

fn main()
{
	// - Parse arguments
	let opts = [
		getopts::optopt("m", "memmap", "Set memory map filename", "FILE"),
		getopts::optopt("t", "types", "Set type list filename", "FILE"),
		];
	let args = match getopts::getopts(::std::os::args().as_slice(), opts)
		{
		Ok(v) => v,
		Err(reason) => fail!(reason.to_string()),
		};
	let typesfile = args.opt_str("types").unwrap_or( String::from_str("types.txt") );
	let mapfile = args.opt_str("memmap").unwrap_or( String::from_str("memorymap.txt") );
	// - Load type list
	let typemap = types::TypeMap::load(typesfile.as_slice());
	// - Load memory map (with files)
	let memory = memory::MemoryState::load(mapfile.as_slice());
	// - Run disassembler
	let cpu = match disasm::cpus::pick("arm")
		{
		Some(x) => x,
		None => fail!("Unknown CPU type"),
		};
	let mut disasm = disasm::Disassembled::new(&memory, cpu);
	disasm.convert_from(0, 0);	// HACK: Address 0, mode 0
	//  Loop until no change in state happens, or a maximum iteration count is hit
	for _ in range(0, MAX_LOOPS)
	{
		let mut cont = false;
		cont |= disasm.convert_queue() > 0;
		if !cont {
			break;
		}
	}
	// - Dump output (JSON with states?)
}

// vim: ft=rust
