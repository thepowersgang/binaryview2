//
//
//
use value::{ValueKnown};

struct Intel32CPU;

pub static CPU_STRUCT: Intel32CPU = Intel32CPU;

impl ::disasm::CPU for Intel32CPU
{
	fn num_regs(&self) -> uint {
		16
	}
	fn prep_state(&self, _state: &mut ::disasm::state::State, _addr: u64, _mode: uint) {
		// X86 doesn't need any pre-instruction prep
	}
	
	fn disassemble(&self, mem: &::memory::MemoryState, addr: u64, mode: uint) -> Result<::disasm::Instruction,()>
	{
		assert!( mode == 0 );
		let val = match mem.read_u8(addr)
			{
			Some(ValueKnown(v)) => v,
			_ => return Err( () )	// Reading from non-concrete memory!
			};
		match val
		{
		_ => {
			error!("Unknown opcode {:02x}", val);
			return Err( () )
			}
		}
	}
	
}


// vim: ft=rust