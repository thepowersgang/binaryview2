//
//
//

struct Intel32CPU;

static CPU_STRUCT: Intel32CPU = Intel32CPU;
static CPU_STRUCT_REF: &'static ::disasm::CPU + 'static = &Intel32CPU as &::disasm::CPU;

impl ::disasm::CPU for Intel32CPU
{
	fn num_regs(&self) -> uint {
		16
	}
	fn disassemble(&self, mem: &::memory::MemoryState, addr: u64, mode: uint) -> Result<::disasm::Instruction,()> {
		fail!("TODO: x86 disassemble");
	}
	fn prep_state(&self, state: &mut ::disasm::state::State, addr: u64, mode: uint) {
		fail!("TODO: x86 prep");
	}
	
}


// vim: ft=rust
