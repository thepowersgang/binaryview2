//
//
//
use std::collections::HashMap;

#[macro_use] extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate num;
extern crate bit_set;

/// Implements the From trait for the provided type, avoiding boilerplate
#[macro_export]
macro_rules! impl_from {
	(@as_item $($i:item)*) => {$($i)*};

	($( $(<($($params:tt)+)>)* From<$src:ty>($v:ident) for $t:ty { $($code:stmt)*} )+) => {
		$(impl_from!{ @as_item 
			impl$(<$($params)+>)* ::std::convert::From<$src> for $t {
				fn from($v: $src) -> $t {
					$($code)*
				}
			}
		})+
	};
}


//mod sortedlist;	// Trait - Provides a sorted list interface to generic types

mod value;	// Value type
mod memory;	// Memory
mod types;	// Type manager
mod disasm;	// Disassembler
//mod analyse;	// Analysis of the disassembled code (to produce more addresses, and get functions)
mod parse;	// Configuration parsing

static MAX_LOOPS: usize = 50;	// Maximum number of passes during disassembly+processing

fn main()
{
	env_logger::init().unwrap();
	let str_args: Vec<_> = ::std::env::args().collect();
	// - Parse arguments
	let mut opts = getopts::Options::new();
	opts.optopt("m", "memmap", "Set memory map filename", "FILE");
	opts.optopt("t", "types", "Set type list filename", "FILE");
	let args = match opts.parse(&str_args[1..])
		{
		Ok(v) => v,
		Err(reason) => panic!("getopts() failed: {}", reason),
		};
	let typesfile = args.opt_str("types").unwrap_or( String::from("types.txt") );
	let mapfile = args.opt_str("memmap").unwrap_or( String::from("memorymap.txt") );
	// - Open input files
	let mut infiles: HashMap<_, _> = args.free.iter().map(|p| {
		let mut s = p.split('=');
		let ident = s.next().unwrap();
		let path = s.next().expect("ERROR: Free arguments should be of the form '<name>=<path>'");
		if let Some(_) = s.next() {
			panic!("ERROR: Free arguments should be of the form '<name>=<path>'");
		}
		let file = match ::std::fs::File::open(&path) {
			Ok(x) => x,
			Err(e) => panic!("ERROR: Unable to open file '{}' for reading. Reason: {}", path, e)
			};
		(String::from(ident), file)
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
		let mut cont = false;
		// - Convert the current queue of "to-process" addresses (jump and call targets)
		cont |= disasm.convert_queue() > 0;
		// - Determine code blocks (and methods)
		cont |= disasm.pass_block_run() > 0;
		// - Acquire clobber lists for methods
		//  > Scan methods from leaf methods first (loops handled somehow?)
		cont |= disasm.pass_callingconv() > 0;
		// - Determine value ranges
		// - Rescan for new addresses to process
		if !cont {
			break;
		}
		pass_count += 1;
	}
	// - Dump output (JSON with states?)
	debug!("TOTALS:");
	debug!(" Pass Count = {}", pass_count);
	debug!(" Instruction Count = {}", disasm.instr_count());
	
	let _ = disasm.dump( &mut WriterWrapper(::std::io::stdout()) );
}

struct WriterWrapper<T: ::std::io::Write>(T);

impl<T: ::std::io::Write> ::std::fmt::Write for WriterWrapper<T>
{
	fn write_str(&mut self, bytes: &str) -> ::std::fmt::Result
	{
		match self.0.write(bytes.as_bytes())
		{
		Ok(_) => Ok( () ),
		Err(_) => Err( ::std::fmt::Error ),
		}
	}
}

// vim: ft=rust
