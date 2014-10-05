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
	
	fn get_region(&self, addr: u64) -> Option<(&Region,uint)> {
		match self.regions.as_slice().binary_search(|r| r.start.cmp(&addr))
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
	pub fn read_u16(&self, addr: u64) -> Option<Value<u16>> {
		self.get_region(addr).map(
			|(a,ofs)| Value::<u16>::concat(a.read_u8(ofs), a.read_u8(ofs+1))
			)
	}
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

// vim: ft=rust
