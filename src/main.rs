//
//
//
#![feature(associated_types)]

mod sortedlist;	// Trait - Provides a sorted list interface to generic types

mod value;	// Value type
mod memory;	// Memory
mod types;	// Type manager
mod disasm;	// Disassembler
//mod analyse;	// Analysis of the disassembled code (to produce more addresses, and get functions)

fn main()
{
	// - Parse arguments
	// - Load type list
	// - Load memory map (with files)
	// - Run disassembler
	let cpu = disasm::cpus::pick("x86");
	// - Dump output (JSON with states?)
}

// vim: ft=rust
