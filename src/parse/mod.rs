//
//
//
use utf8reader::UTF8Reader;

mod lexer;

macro_rules! assert_token{
	($pat:pat => $ret:expr : $tok:expr) => (match $tok { $pat => $ret, tok @ _ => return Err( format!("Unexpected {}, expected {}", tok, stringify!($pat)) )});
	($pat:pat : $tok:expr) => (assert_token!($pat => () : $tok));
}

pub fn get_tok(lex: &mut lexer::Lexer) -> Result<lexer::Token,String>
{
	lex.get_token().map_err(|e|format!("Lex Error: {}", e))
}

pub fn parse_memorymap(state: &mut ::memory::MemoryState, path: &str) -> Result<(),String>
{
	let fp = ::std::io::File::open(&::std::path::Path::new(path)).unwrap();
	let mut reader = UTF8Reader::new(fp);
	let mut lex = lexer::Lexer::new( &mut reader );
	
	// read symbol, select action
	loop
	{
		match try!(get_tok(&mut lex))
		{
		lexer::TokIdent(ident) => match ident.as_slice()
			{
			"RAM" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let size = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add RAM {:#x}+{:#x}", addr, size);
				},
			"MMIO" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let size = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add MMIO {:#x}+{:#x}", addr, size);
				},
			"ROM" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let file = assert_token!( lexer::TokIdent(s) => s : try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add ROM {:#x} ident {}", addr, file);
				},
			"ENTRY" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let mode = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add entrypoint {:#x} mode={}", addr, mode);
				},
			"METHOD" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let name = assert_token!( lexer::TokIdent(s) => s : try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokParenOpen : try!(get_tok(&mut lex)) );
				let mut args = Vec::<::types::Type>::new();
				assert_token!( lexer::TokParenClose : try!(get_tok(&mut lex)) );
				let ret_type = try!( parse_type(&mut lex) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add method {} at {:#x}, args: {}, ret: {}", name, addr, args, ret_type);
				},
			"STATIC" => {
				let addr = assert_token!( lexer::TokInteger(i) => i : try!(get_tok(&mut lex)) );
				let name = assert_token!( lexer::TokIdent(s) => s : try!(get_tok(&mut lex)) );
				let val_type = try!( parse_type(&mut lex) );
				assert_token!( lexer::TokNewline : try!(get_tok(&mut lex)) );
				debug!("Add static {} at {:#x}, type: {}", name, addr, val_type);
				},
			_ => return Err( format!("Unknown attribute in memory map '{}'", ident) ),
			},
		lexer::TokEof => break,
		lexer::TokNewline => continue,
		tok @ _ => {
			return Err( format!("Unexpected {}, expected TokIdent or TokEOF", tok) );
			}
		}
	}
	
	//  > Memory mapped items
	//memory.add_ram(0x02000000, 0x40000);
	//  > Entrypoints
	//  > Symbol Table
	//  > Override list
	
	Ok( () )
}

fn parse_type(lex: &mut lexer::Lexer) -> Result<::types::Type,String>
{
	let mut ptrdepth = 0;
	loop
	{
		match try!(get_tok(lex))
		{
		lexer::TokStar => {
			ptrdepth += 1;
			}
		tok @ _ => {
			lex.put_back(tok);
			break;
			}
		}
	}
	
	// TODO: Arrays
	
	let roottype = assert_token!( lexer::TokIdent(i) => i : try!(get_tok(lex)) );
	let inner = match roottype.as_slice()
		{
		"void" => ::types::TypeInt(0),
		"i8"  => ::types::TypeInt(1),
		"i16" => ::types::TypeInt(2),
		"i32" => ::types::TypeInt(3),
		_ => ::types::TypeStruct(roottype)
		};
	
	Ok(if ptrdepth > 0 {
			::types::TypePointer(ptrdepth, inner)
		}
		else {
			::types::TypeLit(inner)
		})
	
}

// vim: ft=rust
