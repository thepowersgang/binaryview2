//
//
//
#![feature(box_syntax)]

#[macro_use] extern crate log;
extern crate getopts;

mod sortedlist;	// Trait - Provides a sorted list interface to generic types

mod value;	// Value type
mod memory;	// Memory
mod types;	// Type manager
mod disasm;	// Disassembler
//mod analyse;	// Analysis of the disassembled code (to produce more addresses, and get functions)
mod parse;	// Configuration parsing

static MAX_LOOPS: usize = 32;	// Maximum number of passes during disassembly+processing

fn main()
{
	let str_args = ::std::os::args();
	// - Parse arguments
	let opts = [
		getopts::optopt("m", "memmap", "Set memory map filename", "FILE"),
		getopts::optopt("t", "types", "Set type list filename", "FILE"),
		];
	let args = match getopts::getopts(&str_args[(1..)], &opts)
		{
		Ok(v) => v,
		Err(reason) => panic!("getopts() failed: {}", reason),
		};
	let typesfile = args.opt_str("types").unwrap_or( String::from_str("types.txt") );
	let mapfile = args.opt_str("memmap").unwrap_or( String::from_str("memorymap.txt") );
	// - Open input files
	let mut infiles: std::collections::HashMap<String,::std::io::File> = args.free.iter().map(|p| {
		let mut s = p.split('=');
		let ident = s.next().unwrap();
		let path = s.next().expect("ERROR: Free arguments should be of the form '<name>=<path>'");
		if let Some(_) = s.next() {
			panic!("ERROR: Free arguments should be of the form '<name>=<path>'");
		}
		let file = match ::std::io::File::open(&::std::path::Path::new(path)) {
			Ok(x) => x,
			Err(e) => panic!("ERROR: Unable to open file '{}' for reading. Reason: {}", path, e)
			};
		(String::from_str(ident), file)
		}).collect();
	
	// ------------------------------------------------------------
	// Load program state
	// ------------------------------------------------------------
	// - Load type list
	let typemap = {
		let mut tmp = types::TypeMap::new();
		::parse::parse_typemap(&mut tmp, &*typesfile).unwrap();
		tmp
		};
	// - Load memory map (includes overrides)
	let mut memory = memory::MemoryState::new();
	let (entrypoints,) = ::parse::parse_memorymap(
		&mut memory,
		&typemap, &mut infiles,
		&*mapfile
		).unwrap();
	// - Select CPU
	// TODO: Obtain CPU type from memory map
	let cpu = match disasm::cpus::pick("arm")
		{
		Some(x) => x,
		None => panic!("Unknown CPU type"),
		};
	// ------------------------------------------------------------
	// Run disassembler
	// ------------------------------------------------------------
	// > Iterate entrypoints, running conversion (and obtaining further addresses to process)
	let mut disasm = disasm::Disassembled::new(&memory, cpu);
	for addr in entrypoints.into_iter()
	{
		disasm.convert_from(addr);
	}
	// > Loop until no change in state happens, or a maximum iteration count is hit
	let mut pass_count = 0;
	while pass_count < MAX_LOOPS
	{
		pass_count += 1;
		
		let mut cont = false;
		// - Convert the current queue of "to-process" addresses (jump and call targets)
		cont |= disasm.convert_queue() > 0;
		// - Determine code blocks (and methods)
		cont |= disasm.pass_blockify() > 0;
		// - Acquire clobber lists for methods
		//  > Scan methods from leaf methods first (loops handled somehow?)
		// - Determine value ranges
		// - Rescan for new addresses to process
		if !cont {
			break;
		}
	}
	// - Dump output (JSON with states?)
	debug!("TOTALS:");
	debug!(" Pass Count = {}", pass_count);
	debug!(" Instruction Count = {}", disasm.instr_count());
	let mut stdout = WriterWrapper(::std::io::stdout());
	disasm.dump( &mut stdout );
}

struct WriterWrapper<T:Writer>(T);

impl<T:Writer> ::std::fmt::Writer for WriterWrapper<T>
{
	fn write_str(&mut self, bytes: &str) -> ::std::fmt::Result
	{
		match self.0.write_str(bytes)
		{
		Ok(_) => Ok( () ),
		Err(_) => Err( ::std::fmt::Error ),
		}
	}
}

// vim: ft=rust
