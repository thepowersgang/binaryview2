//
//
//
use std::collections::HashMap;
//use std::collections::TreeMap;

pub struct TypeMap
{
	structs: HashMap<String,Struct>
}

#[deriving(Show)]
pub enum InnerType
{
	TypeInt(u8),
	TypeStruct(String),
	TypeString(String),
}

#[deriving(Show)]
pub enum Type
{
	TypeLit(InnerType),
	TypePointer(u8,InnerType),
}

#[deriving(Show)]
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
		match self.structs.entry(String::from_str(name))
		{
		::std::collections::hashmap::Occupied(_) => Err( () ),
		::std::collections::hashmap::Vacant(e) => {
			Ok( e.set( Struct::new() ) )
			},
		}
	}
	
	pub fn get_type_by_name(&self, name: &str) -> Result<InnerType,()>
	{
		//debug!("self.structs = {}", self.structs);
		match name
		{
		"void" => Ok( ::types::TypeInt(0) ),
		"i8"  => Ok( ::types::TypeInt(1) ),
		"i16" => Ok( ::types::TypeInt(2) ),
		"i32" => Ok( ::types::TypeInt(3) ),
		"u8"  => Ok( ::types::TypeInt(1) ),
		"u16" => Ok( ::types::TypeInt(2) ),
		"u32" => Ok( ::types::TypeInt(3) ),
		_ => {
			let key = String::from_str(name);
			match self.structs.find(&key)
			{
			Some(_) => Ok( ::types::TypeStruct(key) ),
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
