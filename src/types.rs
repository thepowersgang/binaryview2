//
//
//
use std::collections::HashMap;
use std::collections::TreeMap;

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

struct Struct
{
	fields: TreeMap<String,Type>,
}


impl TypeMap
{
	pub fn load(path: &str) -> TypeMap
	{
		TypeMap {
			structs: HashMap::new(),
		}
	}
}

// vim: ft=rust
