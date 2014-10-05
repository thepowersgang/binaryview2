//
//
//
#![feature(associated_types)]
#![feature(phase)]
#![feature(macro_rules)]

#[phase(plugin,link)] extern crate log;
extern crate getopts;
extern crate utf8reader;	// 'thepowersgang/rust-utf8reader' - Provides an inline UTF-8 decoder

mod sortedlist;	// Trait - Provides a sorted list interface to generic types

mod value;	// Value type
mod memory;	// Memory
mod types;	// Type manager
mod disasm;	// Disassembler
//mod analyse;	// Analysis of the disassembled code (to produce more addresses, and get functions)
mod parse;	// Configuration parsing

static MAX_LOOPS: uint = 16;	// Maximum number of passes during disassembly+processing

fn main()
{
	// - Parse arguments
	let opts = [
		getopts::optopt("m", "memmap", "Set memory map filename", "FILE"),
		getopts::optopt("t", "types", "Set type list filename", "FILE"),
		];
	let args = match getopts::getopts(::std::os::args().slice_from(1), opts)
		{
		Ok(v) => v,
		Err(reason) => fail!(reason.to_string()),
		};
	
	let typesfile = args.opt_str("types").unwrap_or( String::from_str("types.txt") );
	let mapfile = args.opt_str("memmap").unwrap_or( String::from_str("memorymap.txt") );
	let mut infiles: std::collections::HashMap<String,::std::io::File> = args.free.iter().map(|p| {
		let mut s = p.as_slice().split('=');
		let ident = s.next().unwrap();
		let path = match s.next() {
			Some(x) => x,
			None => fail!("ERROR: Free arguments should be of the form '<name>=<path>', got '{}'", p),
			};
		let file = match ::std::io::File::open(&::std::path::Path::new(path)) {
			Ok(x) => x,
			Err(e) => fail!("ERROR: Unable to open file '{}' for reading. Reason: {}", path, e)
			};
		(String::from_str(ident), file)
		}).collect();
	
	// - Load type list
	let typemap = {
		let mut tmp = types::TypeMap::new();
		::parse::parse_typemap(&mut tmp, typesfile.as_slice()).unwrap();
		tmp
		};
	// - Load memory map (includes overrides)
	let mut memory = {
		let mut tmp = memory::MemoryState::new();
		::parse::parse_memorymap(&mut tmp, &typemap, &mut infiles, mapfile.as_slice()).unwrap();
		tmp
		};
	// - Run disassembler
	let cpu = match disasm::cpus::pick("arm")
		{
		Some(x) => x,
		None => fail!("Unknown CPU type"),
		};
	let mut disasm = disasm::Disassembled::new(&memory, cpu);
	disasm.convert_from(0, 0);	// HACK: Address 0, mode 0
	disasm.convert_from(0x08000000, 0);	// HACK: Address 0, mode 0
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
