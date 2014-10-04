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
		RegionROM(ref data) => Value::fixed(data[ofs]),
		RegionRAM(ref data) => data[ofs].clone(),
		}
	}
}

impl MemoryState
{
	pub fn load(path: &str) -> MemoryState {
		MemoryState {
			endian_big: false,
			regions: Vec::new(),
		}
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
