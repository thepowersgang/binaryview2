//
//
//

mod x86;

pub fn pick(name: &str) -> Option<&'static super::CPU>
{
	match name
	{
	"x86" => Some( x86::CPU_STRUCT_REF ),
	_ => None
	}
}

// vim: ft=rust
