//
//
//

mod x86;
mod arm;

pub fn pick(name: &str) -> Option<&'static (super::CPU + 'static)>
{
	match name
	{
	"x86" => Some( &x86::CPU_STRUCT as &super::CPU ),
	"arm" => Some( &arm::CPU_STRUCT as &super::CPU ),
	_ => None
	}
}

// vim: ft=rust
