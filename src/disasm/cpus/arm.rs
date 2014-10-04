//
//
//
use value::{Value,ValueKnown};

struct ArmCpu;

pub static CPU_STRUCT: ArmCpu = ArmCpu;

impl ::disasm::CPU for ArmCpu
{
	fn num_regs(&self) -> uint {
		16
	}
	fn prep_state(&self, state: &mut ::disasm::state::State, addr: u64, mode: uint) {
		let pc_val = match mode
			{
			0 => addr + 8,  	// ARM mode
			1 => addr + 4 + 1,	// THUMB mode
			_ => fail!("Invalid ARM mode"),
			};
		state.set( ::disasm::ParamTrueReg(15), Value::fixed(pc_val) );
	}
	
	fn disassemble(&self, mem: &::memory::MemoryState, addr: u64, mode: uint) -> Result<::disasm::Instruction,()>
	{
		match mode
		{
		0 => disassemble_arm(mem, addr),
		1 => disassemble_thumb(mem, addr),
		_ => fail!("Invalid ARM mode"),
		}
	}
	
}

fn disassemble_arm(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let val = match mem.read_u32(addr)
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

fn disassemble_thumb(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let val = match mem.read_u16(addr)
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


// vim: ft=rust
