// BinaryView2
// - by John Hodge (thePowersGang)
//
// utf8reader.rs
// - Reads a stream of UTF-8 encoded codepoints from a "Reader"
use std::io::IoResult;

static BADCHAR: char = '\uFFFD';

pub struct UTF8Reader<T: Reader>
{
	stream: T,
}

impl<T:Reader> UTF8Reader<T>
{
	pub fn new(reader: T) -> UTF8Reader<T>
	{
		UTF8Reader {
			stream: reader,
		}
	}
	
	/// Read a single codepoint from the stream.
	/// On an encoding error, it returns '\uFFFD' (the unicode replacement character)
	pub fn getc(&mut self) -> IoResult<char>
	{
		let ch1 = try!(self.stream.read_byte());
		if ch1 & 0xC0 == 0x80 {
			return Ok( BADCHAR )
		}
		if ch1 & 0x80 == 0x00
		{
			// Single-byte
			Ok(ch1 as char)
		}
		else if ch1 & 0xE0 == 0xC0
		{
			// Two-byte sequence
			let ch2 = try!(self.stream.read_byte());
			if ch2 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			
			let ret = (ch1 & 0x1F << 6) | (ch2 & 0x3F << 0);
			Ok( ret as char )
		}
		else if ch1 & 0xF0 == 0xE0
		{
			// Three-byte sequence
			let ch2 = try!(self.stream.read_byte());
			if ch2 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			let ch3 = try!(self.stream.read_byte());
			if ch3 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			
			let ret = (ch1 & 0x0F << 12) | (ch2 & 0x3F << 6) | (ch3 & 0x3F << 0);
			Ok( ret as char )
		}
		else if ch1 & 0xF8 == 0xF0
		{
			// Four-byte sequence
			let ch2 = try!(self.stream.read_byte());
			if ch2 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			let ch3 = try!(self.stream.read_byte());
			if ch3 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			let ch4 = try!(self.stream.read_byte());
			if ch4 & 0xC0 != 0x80 {
				return Ok( BADCHAR );
			}
			
			let ret = (ch1 & 0x07 << 18) | (ch2 & 0x3F << 12) | (ch3 & 0x3F << 6) | (ch4 & 0x3F << 0);
			Ok( ret as char )
		}
		else
		{
			// More than four bytes is invalid
			Ok( BADCHAR )
		}
	}
}

impl<T:Reader> Iterator<IoResult<char>> for UTF8Reader<T>
{
	fn next(&mut self) -> Option<IoResult<char>>
	{
		// Get result from decoder
		match self.getc()
		{
		// - All good, return a character
		Ok(c) => Some( Ok(c) ),
		// - Error, check if it's EOF
		Err(e) => match e.kind {
			// Return 'None' on EOF (end of stream)
			::std::io::EndOfFile => None,
			_ => Some( Err( e ) ),
			}
		}
	}
}

// vim: ft=rust
