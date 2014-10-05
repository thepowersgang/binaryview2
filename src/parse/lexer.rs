// BinaryView2
// - By John Hodge (thePowersGang)
//
// lexer.rs
// - Common lexer used for config files
//
// TODO: Make this even more generic, using a syntax extension to provide format description
extern crate libc;

use std::io::IoResult;

#[deriving(Show)]
pub enum Token
{
	TokEof,
	TokNewline,
	TokStar,
	TokParenOpen,	TokParenClose,
	TokSquareOpen,	TokSquareClose,
	TokBraceOpen,	TokBraceClose,
	TokInteger(u64),
	TokIdent(String),
	TokString(String),
	TokLineComment(String),
}

type LexResult<T> = Result<T,()>;

/// Core lexer type
pub struct Lexer<'r>
{
	instream: &'r mut Iterator<IoResult<char>>+'r,
	lastc: Option<char>,
	saved_tok: Option<Token>,
}

impl<'a> Lexer<'a>
{
	pub fn new<'r>(instream: &'r mut Iterator<IoResult<char>>) -> Lexer<'r> {
		Lexer {
			instream: instream,
			lastc: None,
			saved_tok: None,
		}
	}
	
	pub fn get_token(&mut self) -> LexResult<Token>
	{
		match self.saved_tok.take()
		{
		None => loop
			{
				match try!(self.gettoken_int())
				{
				TokLineComment(_) => continue,
				tok @ _ => return Ok(tok),
				}
			},
		Some(tok) => {
			Ok(tok)
			}
		}
	}
	pub fn put_back(&mut self, tok: Token)
	{
		self.saved_tok = Some(tok);
	}

	fn getc(&mut self) -> LexResult<char>
	{
		match self.lastc
		{
		Some(x) => {
			self.lastc = None;
			Ok( x )
			},
		None => match self.instream.next()
			{
			Some(x) => Ok( match x { Ok(a)=>a,Err(_)=>return Err( () )} ),
			None => Ok( '\0' )
			},
		}
	}
	fn ungetc(&mut self, c: char)
	{
		self.lastc = Some(c);
	}

	// Read and return the rest of the line
	// - Eof is converted to return value
	fn read_to_eol(&mut self) -> LexResult<String>
	{
		let mut ret = String::new();
		loop
		{
			let ch = try!(self.getc());
			if ch == '\n' || ch == '\0' {
				self.ungetc(ch);
				break;
			}
			ret.push( ch );
		}
		return Ok(ret);
	}
	// Read and return a sequence of "identifier" characters
	fn read_ident(&mut self) -> LexResult<String>
	{
		let mut name = String::new();
		loop
		{
			let ch = try!(self.getc());
			if !(isalnum(ch) || ch == '_') {
				self.ungetc(ch);
				break;
			}
			name.push( ch );
		}
		return Ok(name);
	}
	fn read_number(&mut self, base: uint) -> LexResult<u64>
	{
		let mut val = 0;
		loop
		{
			let ch = try!(self.getc());
			match ch.to_digit(base) {
			Some(d) => {
				val *= base as u64;
				val += d as u64
				},
			None => {
				self.ungetc(ch);
				break;
				}
			}
		}
		return Ok(val);
	}
	
	fn gettoken_int(&mut self) -> LexResult<Token>
	{
		loop {
                        let ch = try!(self.getc());
                        if ch == '\n' || ch == '\0' || !isspace(ch) {
				self.ungetc(ch);
                                break;
                        }
                }
		
		let ch = try!(self.getc());
		let ret = match ch
		{
		'\0' => TokEof,
		'\n' => TokNewline,
		'#' => TokLineComment( try!(self.read_to_eol()) ),
		'{' => TokBraceOpen,	'}' => TokBraceClose,
		'[' => TokSquareOpen,	']' => TokSquareClose,
		'(' => TokParenOpen,	')' => TokParenClose,
		'0' => TokInteger( {
			let ch2 = try!(self.getc());
			match ch2 {
			'1'...'7' => {
				self.ungetc(ch2);
				try!(self.read_number( 8))
				},
			'x' => try!(self.read_number(16)),
			'b' => try!(self.read_number( 2)),
			_ => {
				self.ungetc(ch2);
				0
				}
			}
			}),
		'1'...'9' => TokInteger( {
			self.ungetc(ch);
			try!(self.read_number(10))
			}),
		'a'...'z'|'A'...'Z'|'_' => {
			self.ungetc(ch);
			let ident = try!(self.read_ident());
			TokIdent( ident )
			},
		_ => {
			error!("Bad character #{} '{}' hit", ch as u32, ch);
			return Err( () )
			}
		};
		
		debug!("Token = {}", ret);
		
		return Ok( ret );
	}
}

fn isspace(ch: char) -> bool {
	unsafe {
		return libc::funcs::c95::ctype::isspace(ch as i32) != 0
	}
}
fn isalnum(ch: char) -> bool {
	unsafe {
		return libc::funcs::c95::ctype::isalnum(ch as i32) != 0
	}
}

// vim: ft=rust
