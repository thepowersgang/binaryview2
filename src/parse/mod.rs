//
//
//
use disasm::CodePtr;

mod lexer;

macro_rules! assert_token{
	($pat:pat , $val:expr , $tok:expr , $name:expr) => (match $tok { $pat => $val, tok @ _ => return Err( format!("Unexpected {:?}, expected {:?}", tok, $name) )});
	(lexer::$pat:ident($val:ident) = $tok:expr) => (assert_token!(lexer::$pat($val) , ($val) , $tok , stringify!($pat)));
	(lexer::$pat:ident = $tok:expr) => (assert_token!(lexer::$pat , () , $tok , stringify!($pat)));
}

pub fn get_tok(lex: &mut lexer::Lexer) -> Result<lexer::Token,String>
{
	lex.get_token().map_err(|e|format!("Lex Error: {:?}", e))
}

/// Parse a memory map file
///
/// \param memory	Memory state, mutated as part of processing
/// \param typemap	Avaliable custom types
/// \param infiles	Map of input files (mutable to allow use of the contained File struct)
/// \param path 	Path to the memory map file
pub fn parse_memorymap(
	memory: &mut ::memory::MemoryState,
	//symbols: &mut ::symbols::Symbols,
	typemap: &::types::TypeMap,
	infiles: &mut ::std::collections::HashMap<String,::std::fs::File>,
	path: &str
	)
	-> Result<(Vec<CodePtr>,),String>
{
	let mut entrypoints = Vec::new();
	let fp = ::std::fs::File::open(path).unwrap();
	let mut reader = ::std::io::BufReader::new(fp);
	let mut lex = lexer::Lexer::new( &mut reader );
	
	// read symbol, select action
	loop
	{
		match try!(get_tok(&mut lex))
		{
		lexer::TokIdent(ident) => match &*ident
			{
			"RAM" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let size = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add RAM {:#x}+{:#x}", addr, size);
				memory.add_ram(addr, size as usize);
				},
			"MMIO" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let size = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add MMIO {:#x}+{:#x}", addr, size);
				memory.add_mmio(addr, size as usize, "");
				},
			"ROM" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let size = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let file_id = assert_token!( lexer::TokIdent(s) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add ROM {:#x}+{:#x} ident {}", addr, size, file_id);
				match infiles.get_mut(&file_id)
				{
				None => return Err( format!("No filename set for ident '{}'", file_id) ),
				Some(file_struct) => {
					memory.add_rom(addr, size as usize, file_struct);
					}
				}
				},
			"ENTRY" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let mode = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add entrypoint {:#x} mode={}", addr, mode);
				entrypoints.push( CodePtr::new(mode as ::disasm::CPUMode, addr) );
				},
			"METHOD" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let name = assert_token!( lexer::TokIdent(s) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokParenOpen = try!(get_tok(&mut lex)) );
				let mut args = Vec::new();
				loop
				{
					match try!(get_tok(&mut lex))
					{
					tok @ lexer::TokParenClose => {
						lex.put_back(tok);
						break;
						},
					lexer::TokIdent(paramname) => {
						assert_token!( lexer::TokColon = try!(get_tok(&mut lex)) );
						let paramtype = try!( parse_type(typemap, &mut lex) );
						args.push( (paramname, paramtype) );
						},
					tok @ _ => return Err( format!("Unexpected '{:?}', expected TokParenClose or TokIdent", tok) )
					}
					
					match try!(get_tok(&mut lex))
					{
					tok @ lexer::TokParenClose => {
						lex.put_back(tok);
						break;
						},
					lexer::TokComma => {},
					tok @ _ => return Err( format!("Unexpected '{:?}', expected TokParenClose or TokComma", tok) )
					}
				}
				assert_token!( lexer::TokParenClose = try!(get_tok(&mut lex)) );
				let ret_type = try!( parse_type(typemap, &mut lex) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add method {} at {:#x}, args: {:?}, ret: {:?}", name, addr, args, ret_type);
				error!("TODO: Add method {}", name);
				},
			"STATIC" => {
				let addr = assert_token!( lexer::TokInteger(i) = try!(get_tok(&mut lex)) );
				let name = assert_token!( lexer::TokIdent(s) = try!(get_tok(&mut lex)) );
				let val_type = try!( parse_type(typemap, &mut lex) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Add static {} at {:#x}, type: {:?}", name, addr, val_type);
				},
			_ => return Err( format!("Unknown attribute in memory map '{}'", ident) ),
			},
		lexer::TokEof => break,
		lexer::TokNewline => continue,
		tok @ _ => {
			return Err( format!("Unexpected {:?}, expected TokIdent or TokEOF", tok) );
			}
		}
	}
	
	//  > Memory mapped items
	//memory.add_ram(0x02000000, 0x40000);
	//  > Entrypoints
	//  > Symbol Table
	//  > Override list
	
	Ok( (entrypoints,) )
}

pub fn parse_typemap(typemap: &mut ::types::TypeMap, path: &str) -> Result<(),String>
{
	let mut reader = ::std::fs::File::open(path).unwrap();
	let mut lex = lexer::Lexer::new( &mut reader );
	
	loop
	{
		match try!(get_tok(&mut lex))
		{
		lexer::TokIdent(ident) => match &*ident
			{
			"STRUCT" => {
				// First line: STRUCT <name> "<format>"
				let name = assert_token!( lexer::TokIdent(s) = try!(get_tok(&mut lex)) );
				let fmt = assert_token!( lexer::TokString(s) = try!(get_tok(&mut lex)) );
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				debug!("Parsing structure '{}' (format = \"{}\")", name, fmt);
				// Fields: <name> <type>
				// - terminated by: END
				let mut flds = Vec::new();
				loop
				{
					let fldname = assert_token!( lexer::TokIdent(s) = try!(get_tok(&mut lex)) );
					if &*fldname == "END" {
						break;
					}
					let fldtype = try!( parse_type(typemap, &mut lex) );
					assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
					flds.push( (fldname, fldtype) );
				}
				assert_token!( lexer::TokNewline = try!(get_tok(&mut lex)) );
				// Create the structure
				let newstruct: &mut _ = match typemap.new_struct(&*name)
					{
					Ok(s) => s,
					Err(_) => return Err( format!("Duplicate definition of structure '{}'", name) ),
					};
				for (fldname, fldtype) in flds.into_iter() {
					newstruct.append_field(fldname, fldtype);
				}
				},
			_ => return Err( format!("Unknown keyword in type list '{}'", ident) ),
			},
		lexer::TokEof => break,
		lexer::TokNewline => continue,
		tok @ _ => {
			return Err( format!("Unexpected {:?}, expected TokIdent or TokEOF", tok) );
			}
		}
	}

	Ok( () )
}

fn parse_type(typemap: &::types::TypeMap, lex: &mut lexer::Lexer) -> Result<::types::Type,String>
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
	
	let roottype = assert_token!( lexer::TokIdent(i) = try!(get_tok(lex)) );
	let inner = match typemap.get_type_by_name( &*roottype )
		{
		Ok(t) => t,
		Err(_) => return Err( format!("Unknown type name '{}'", roottype) ),
		};
	
	Ok(if ptrdepth > 0 {
			::types::Type::Pointer(ptrdepth, inner)
		}
		else {
			::types::Type::Lit(inner)
		})
	
}

// vim: ft=rust
