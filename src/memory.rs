//
//
//
use value::Value;
use std::cmp::Ordering;
use std::io::{Read,Seek};

/// Memory region type
enum RegionType
{
	ROM(Vec<u8>),
	RAM(Vec<Value<u8>>),
	MMIO(String),
}

struct Region
{
	start: u64,
	size: usize,
	data: RegionType
}

pub struct MemoryState
{
	endian_big: bool,
	regions: Vec<Region>,
}

pub trait MemoryStateAccess:
	::value::ValueType
{
	fn read(&MemoryState, addr: u64) -> Option<Value<Self>>;
	fn write(&mut MemoryState, addr: u64, val: Value<Self>);
}

impl Region
{
	pub fn read_u8(&self, ofs: usize) -> Value<u8> {
		match self.data
		{
		RegionType::ROM(ref data) => Value::known(data[ofs % self.size]),	// ROMs wrap
		RegionType::RAM(ref data) => data[ofs].clone(),
		RegionType::MMIO(_) => Value::unknown(),
		}
	}
	pub fn read_u16_le(&self, ofs: usize) -> Value<u16> {
		Value::concat(self.read_u8(ofs+0), self.read_u8(ofs+1))
	}
	pub fn read_u32_le(&self, ofs: usize) -> Value<u32> {
		Value::concat(self.read_u16_le(ofs+0), self.read_u16_le(ofs+2))
	}
	pub fn read_u16_be(&self, ofs: usize) -> Value<u16> {
		Value::concat(self.read_u8(ofs+1), self.read_u8(ofs+0))
	}
	pub fn read_u32_be(&self, ofs: usize) -> Value<u32> {
		Value::concat(self.read_u16_be(ofs+2), self.read_u16_be(ofs+0))
	}
	
	/// Compare, treating inside as equal
	fn cmp_inner(&self, addr: u64) -> Ordering
	{
		if addr < self.start {
			Ordering::Greater
		}
		else if addr >= self.start + self.size as u64 {
			Ordering::Less
		}
		else {
			Ordering::Equal
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
	
	fn add_region(&mut self, base: u64, size: usize, data: RegionType)
	{
		let pos = match self.regions.binary_search_by(|r| r.start.cmp(&base))
			{
			Ok(_) => panic!("region overlap"),
			Err(idx) => {
				if idx > 0 && base < self.regions[idx-1].start + self.regions[idx-1].size as u64 {
					panic!("region overlap");
				}
				if idx < self.regions.len() && base + size as u64 > self.regions[idx].start {
					panic!("region overlap");
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
	pub fn add_rom(&mut self, base: u64, size: usize, file: &mut ::std::fs::File)
	{
		// The ROM repeats as many times as nessesary to reach the stated size
		let filesize = file.metadata().unwrap().len();
		
		// 1. 'filesize' must be a divisor of 'size'
		if size as u64 / filesize * filesize != size as u64 {
			panic!("Unable to map ROM at {:#x}, provided file doesn't fit neatly", base);
		}
		
		// 2. Load data!
		// - Wrapping is handled in Region::read()
		file.seek( ::std::io::SeekFrom::Start(0) ).unwrap();
		let mut data = Vec::new();
		file.read_to_end(&mut data).unwrap();
		self.add_region(base, size, RegionType::ROM(data));
		debug!("Add ROM {:#x}+{:#x}", base, size);
	}
	pub fn add_ram(&mut self, base: u64, size: usize)
	{
		self.add_region(base, size, RegionType::RAM(::std::iter::repeat(Value::unknown()).take(size).collect()));
		debug!("Add RAM {:#x}+{:#x}", base, size);
	}
	pub fn add_mmio(&mut self, base: u64, size: usize, class: &str)
	{
		self.add_region(base, size, RegionType::MMIO(String::from(class)));
		debug!("Add MMIO {:#x}+{:#x} \"{}\"", base, size, class);
	}
	
	
	/// Get the region corresponding to a given address
	fn get_region(&self, addr: u64) -> Option<(&Region,usize)> {
		match self.regions.binary_search_by(|r| r.cmp_inner(addr))
		{
		Ok(idx) => {
			let r = &self.regions[idx];
			if addr - r.start >= r.size as u64 {
				None
			}
			else {
				Some( (r, (addr - r.start) as usize) )
			}
			},
		Err(_) => {
			None
			},
		}
	}
	pub fn read_u8(&self, addr: u64) -> Option<Value<u8>> {
		self.get_region(addr).map( |(a,ofs)| a.read_u8(ofs) )
	}
	/// Read two bytes (from the same region)
	pub fn read_u16(&self, addr: u64) -> Option<Value<u16>> {
		self.get_region(addr).map( |(a,ofs)| if self.endian_big { a.read_u16_be(ofs) } else { a.read_u16_le(ofs) } )
	}
	/// Read four bytes (from the same region)
	pub fn read_u32(&self, addr: u64) -> Option<Value<u32>> {
		self.get_region(addr).map( |(a,ofs)| if self.endian_big { a.read_u32_be(ofs) } else { a.read_u32_le(ofs) } )
	}
	/// Read eight bytes (from the same region)
	pub fn read_u64(&self, addr: u64) -> Option<Value<u64>> {
		self.get_region(addr).map(
			|(a,ofs)|
				if self.endian_big {
					Value::concat( a.read_u32_be(ofs+4), a.read_u32_be(ofs+0) )
				} else {
					Value::concat( a.read_u32_le(ofs+0), a.read_u32_le(ofs+4) )
				}
			)
	}
	
	pub fn write_u8(&self, addr: u64, val: Value<u8>) {
		panic!("TODO: MemoryState.write_u8(addr={:#x},val={:?})", addr, val);
	}
	pub fn write_u16(&self, addr: u64, val: Value<u16>) {
		panic!("TODO: MemoryState.write_u16(addr={:#x},val={:?})", addr, val);
	}
	pub fn write_u32(&self, addr: u64, val: Value<u32>) {
		panic!("TODO: MemoryState.write_u32(addr={:#x},val={:?})", addr, val);
	}
	pub fn write_u64(&self, addr: u64, val: Value<u64>) {
		panic!("TODO: MemoryState.write_u64(addr={:#x},val={:?})", addr, val);
	}
}

impl MemoryStateAccess for u8
{
	fn read(mem: &MemoryState, addr: u64) -> Option<Value<u8>>
	{
		mem.read_u8(addr)
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u8>)
	{
		mem.write_u8(addr, val);
	}
}

impl MemoryStateAccess for u16
{
	fn read(mem: &MemoryState, addr: u64) -> Option<Value<u16>>
	{
		mem.read_u16(addr)
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u16>)
	{
		mem.write_u16(addr, val);
	}
}

impl MemoryStateAccess for u32
{
	fn read(mem: &MemoryState, addr: u64) -> Option<Value<u32>>
	{
		mem.read_u32(addr)
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u32>)
	{
		mem.write_u32(addr, val);
	}
}

impl MemoryStateAccess for u64
{
	fn read(mem: &MemoryState, addr: u64) -> Option<Value<u64>>
	{
		mem.read_u64(addr)
	}
	fn write(mem: &mut MemoryState, addr: u64, val: Value<u64>)
	{
		mem.write_u64(addr, val);
	}
}

// vim: ft=rust
