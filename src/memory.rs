//
//
//
use value::Value;
use std::slice::{Found,NotFound};

/// Memory region type
enum RegionType
{
	RegionROM(Vec<u8>),
	RegionRAM(Vec<Value<u8>>),
	RegionMMIO(String),
}

struct Region
{
	start: u64,
	size: uint,
	data: RegionType
}

pub struct MemoryState
{
	endian_big: bool,
	regions: Vec<Region>,
}

pub trait MemoryStateAccess
{
	fn read(&MemoryState, addr: u64) -> Value<Self>;
	fn write(&mut MemoryState, addr: u64, val: Value<Self>);
}

impl Region
{
	pub fn read_u8(&self, ofs: uint) -> Value<u8> {
		match self.data
		{
		RegionROM(ref data) => Value::fixed(data[ofs % self.size]),	// ROMs wrap
		RegionRAM(ref data) => data[ofs].clone(),
		RegionMMIO(_) => Value::unknown(),
		}
	}
	
	/// Compare, treating inside as equal
	fn cmp_inner(&self, addr: u64) -> Ordering
	{
		if addr < self.start {
			Greater
		}
		else if addr >= self.start + self.size as u64 {
			Less
		}
		else {
			Equal
		}
	}
}

impl MemoryState
{
	pub fn new() -> MemoryState {
		MemoryState {
			endian_big: false,
			regions: Vec::new(),
		}
	}
	
	fn add_region(&mut self, base: u64, size: uint, data: RegionType)
	{
		let pos = match self.regions.as_slice().binary_search(|r| r.start.cmp(&base))
			{
			Found(_) => fail!("region overlap"),
			NotFound(idx) => {
				if idx > 0 && base < self.regions[idx-1].start + self.regions[idx-1].size as u64 {
					fail!("region overlap");
				}
				if idx < self.regions.len() && base + size as u64 > self.regions[idx].start {
					fail!("region overlap");
				}
				idx
				}
			};
		self.regions.insert(pos, Region {
			start: base,
			size: size,
			data: data
			});
	}
	
	/// Load fixed memory from a file
	pub fn add_rom(&mut self, base: u64, size: uint, file: &mut ::std::io::File)
	{
		// The ROM repeats as many times as nessesary to reach the stated size
		file.seek(0, ::std::io::SeekEnd).unwrap();
		let filesize = file.tell().unwrap();
		
		// 1. 'filesize' must be a divisor of 'size'
		if size as u64 / filesize * filesize != size as u64 {
			fail!("Unable to map ROM at {:#x}, provided file doesn't fit neatly", base);
		}
		
		// 2. Load data!
		// - Wrapping is handled in Region::read()
		file.seek(0, ::std::io::SeekSet).unwrap();
		self.add_region(base, size, RegionROM(file.read_to_end().unwrap()));
		debug!("Add ROM {:#x}+{:#x}", base, size);
	}
	pub fn add_ram(&mut self, base: u64, size: uint)
	{
		self.add_region(base, size, RegionRAM(Vec::from_elem(size, Value::unknown())));
		debug!("Add RAM {:#x}+{:#x}", base, size);
	}
	pub fn add_mmio(&mut self, base: u64, size: uint, class: &str)
	{
		self.add_region(base, size, RegionMMIO(String::from_str(class)));
		debug!("Add MMIO {:#x}+{:#x} \"{}\"", base, size, class);
	}
	
	
	/// Get the region corresponding to a given address
	fn get_region(&self, addr: u64) -> Option<(&Region,uint)> {
		match self.regions.as_slice().binary_search(|r| r.cmp_inner(addr))
		{
		Found(idx) => {
			let r = &self.regions[idx];
			if addr - r.start >= r.size as u64 {
				None
			}
			else {
				Some( (r, (addr - r.start) as uint) )
			}
			},
		NotFound(_) => {
			None
			},
		}
	}
	pub fn read_u8(&self, addr: u64) -> Option<Value<u8>> {
		self.get_region(addr).map(|(a,ofs)| a.read_u8(ofs))
	}
	/// Read two bytes (from the same region)
	pub fn read_u16(&self, addr: u64) -> Option<Value<u16>> {
		self.get_region(addr).map(
			|(a,ofs)| Value::<u16>::concat(a.read_u8(ofs), a.read_u8(ofs+1))
			)
	}
	/// Read four bytes (from the same region)
	pub fn read_u32(&self, addr: u64) -> Option<Value<u32>> {
		self.get_region(addr).map(
			|(a,ofs)|
				Value::concat(
					Value::<u16>::concat(a.read_u8(ofs+0), a.read_u8(ofs+1)),
					Value::<u16>::concat(a.read_u8(ofs+2), a.read_u8(ofs+3))
					)
			)
	}
}

impl MemoryStateAccess for u8
{
	fn read(mem: &MemoryState, addr: u64) -> Value<u8>
	{
		mem.read_u8(addr).unwrap_or(Value::unknown())
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u8>)
	{
		unimplemented!();
	}
}

impl MemoryStateAccess for u16
{
	fn read(mem: &MemoryState, addr: u64) -> Value<u16>
	{
		mem.read_u16(addr).unwrap_or(Value::unknown())
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u16>)
	{
		unimplemented!();
	}
}

impl MemoryStateAccess for u32
{
	fn read(mem: &MemoryState, addr: u64) -> Value<u32>
	{
		mem.read_u32(addr).unwrap_or(Value::unknown())
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u32>)
	{
		unimplemented!();
	}
}

// vim: ft=rust
