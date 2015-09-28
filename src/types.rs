//
//
//
use std::collections::HashMap;
//use std::collections::TreeMap;

pub struct TypeMap
{
	structs: HashMap<String,Struct>
}

#[derive(Debug)]
pub enum InnerType
{
	Int(u8),
	Struct(String),	// structure name
	// TODO: Character sets (needed for dumping Pokemon)
	//String(String),	// character set name
}

#[derive(Debug)]
pub enum Type
{
	Lit(InnerType),
	Pointer(u8,InnerType),
}

#[derive(Debug)]
struct Struct
{
	fields: Vec< (String,Type) >,
}


impl TypeMap
{
	pub fn new() -> TypeMap
	{
		TypeMap {
			structs: HashMap::new(),
		}
	}
	
	pub fn new_struct(&mut self, name: &str) -> Result<&mut Struct,()>
	{
		use std::collections::hash_map::Entry;
		match self.structs.entry(String::from(name))
		{
		Entry::Occupied(_) => Err( () ),
		Entry::Vacant(e) => Ok( e.insert( Struct::new() ) ),
		}
	}
	
	pub fn get_type_by_name(&self, name: &str) -> Result<InnerType,()>
	{
		//debug!("self.structs = {}", self.structs);
		match name
		{
		"void" => Ok( InnerType::Int(0) ),
		"i8"  => Ok( InnerType::Int(1) ),
		"i16" => Ok( InnerType::Int(2) ),
		"i32" => Ok( InnerType::Int(3) ),
		"u8"  => Ok( InnerType::Int(1) ),
		"u16" => Ok( InnerType::Int(2) ),
		"u32" => Ok( InnerType::Int(3) ),
		_ => {
			match self.structs.get(name)
			{
			Some(_) => Ok( InnerType::Struct( String::from(name) ) ),
			None => Err( () ),
			}
			}
		}
	}
}

impl Struct
{
	fn new() -> Struct {
		Struct {
			fields: Vec::new(),
		}
	}
	pub fn append_field(&mut self, name: String, fldtype: Type)
	{
		self.fields.push( (name, fldtype) );
	}
}

// vim: ft=rust
