// BinaryView2
// - by John Hodge (thePowersGang)
//
// disam/cpus/arm.rs
// - Recent ARM CPU disassembly (written against ARMv5)
use value::{Value,ValueKnown};
use disasm::common_instrs;
use disasm::{Instruction,InstructionClass};
use disasm::{InstrParam,ParamImmediate,ParamTrueReg};

struct ArmCpu;

#[repr(C)]
enum SystemRegisters
{
	SRegCPSR = 0,
	SRegSPSR = 1,
}

pub static CPU_STRUCT: ArmCpu = ArmCpu;

impl ::disasm::CPU for ArmCpu
{
	fn num_regs(&self) -> uint {
		16
	}
	fn prep_state(&self, state: &mut ::disasm::state::State, addr: u64, mode: uint) {
		let pc_val = match mode
			{
			0 => addr + 8,		// ARM mode
			1 => addr + 4 + 1,	// THUMB mode
			_ => fail!("Invalid ARM mode"),
			};
		state.set( ::disasm::ParamTrueReg(15), Value::known(pc_val) );
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

/// Disassemble code in ARM mode (32-bits per instruction)
fn disassemble_arm(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let word = match mem.read_u32(addr)
		{
		Some(ValueKnown(v)) => v,
		Some(_) => {
			error!("Disassembling non-concrete memory");
			return Err( () );
			},
		None => {
			error!("Disassembling unmapped memory");
			return Err( () );
			}
		};

	let ccode = (word >> 28) as u8;
	if ccode == 0xF {
		error!("TODO: Unconditional instructions");
		return Err( () );
	}
	let op = ((word>>20 & 0xFF) << 4) | (word>>4 & 0xf);
	Ok( match op
	{
        0x120 => {	// mov CPSR, Rn
		Instruction::new(
			4, ccode,
			&instrs::SET_SREG,
			vec![
				ParamImmediate( SRegCPSR as u64 ),
				reg(word, 0),
				ParamImmediate( match (word>>20)&3 { 0=>0,1=>0,2=>0,_=>0 } ),
				]
			)
		},
	// Branch+Exchange Register
	0x121 => Instruction::new( 4, ccode, &instrs::BX, vec![ reg(word, 0) ] ),
	// Branch+Link+Exchange
	0x123 => Instruction::new( 4, ccode, &instrs::BLX, vec![ reg(word, 0) ] ),
	// Logical-Shift-Left (and Reg/Reg Move)
	0x1A0 => {
		let amt = (word >> 7) & 31;
		if amt == 0 {
			Instruction::new( 4, ccode, &common_instrs::MOVE, vec![
				reg(word,12), reg(word,0)
				] )
		}
		else {
			Instruction::new( 4, ccode, &common_instrs::SHL, vec![
				reg(word,12), reg(word,0), ParamImmediate(amt as u64)
				] )
		}
		},
	// Logical-Shift-Left
	0x1A1 => Instruction::new( 4, ccode, &common_instrs::SHL, vec![
			reg(word,12), reg(word,0), reg(word,8)
			]
		),
	// Add (Register, Immediate)
	0x280 ... 0x28F => Instruction::new( 4, ccode, &common_instrs::ADD, vec![
		reg(word, 12), reg(word, 16),
		ParamImmediate( expand_imm_arm(word & 0xFFF) ),
		]
		),
	0x3A0 ... 0x3BF => {
		// Mov Rd, immediate
		let Rd = ((word >> 12) & 0xF) as u8;
		if Rd == 15 {
			// TODO: Handle moving to R15 (aka PC)
			fail!("TODO: Handle move immediate to PC");
		}
		Instruction::new(
			4, ccode,
			&common_instrs::MOVE as &InstructionClass,
			vec![
				ParamTrueReg( Rd ),
				ParamImmediate( expand_imm_arm(word & 0xFFF) ),
				]
			)
		},
	0x580 ... 0x58F => Instruction::new(
		4, ccode,
		&common_instrs::STORE_OFS as &InstructionClass,
		vec![
			ParamTrueReg( ((word>>12)&0xF) as u8 ),
			ParamTrueReg( ((word>>16)&0xF) as u8 ),
			ParamImmediate( sign_extend(12, word & 0xFFF) ),
			]
		),
	// LDR (imm/lit)
	0x590 ... 0x59F => Instruction::new(
		4, ccode,
		&common_instrs::LOAD_OFS as &InstructionClass,
		vec![
			ParamTrueReg( ((word>>12)&0xF) as u8 ),
			ParamTrueReg( ((word>>16)&0xF) as u8 ),
			ParamImmediate( sign_extend(12, word & 0xFFF) ),
			]
		),
	0xA00 ... 0xAFF => {
		// Jump to Address+opr*4+8
		Instruction::new(
			4,
			ccode,
			&common_instrs::JUMP as &InstructionClass,
			vec![ ParamImmediate(addr + 8 + sign_extend(24, word & 0xFFFFFF) * 4), ]
			)
		},
	0xB00 ... 0xBFF => {
		// Branch+Link (call) Address+opr*4+8
		Instruction::new(
			4,
			ccode,
			&common_instrs::CALL as &InstructionClass,
			vec![ ParamImmediate(addr + 8 + sign_extend(24, word & 0xFFFFFF) * 4), ]
			)
		},
	_ => {
		error!("Unknown opcode {:08x} (op={:03x})", word, op);
		return Err( () )
		}
	})
}

/// Disassemble in THUMB mode
fn disassemble_thumb(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let val = match mem.read_u16(addr)
		{
		Some(ValueKnown(v)) => v,
		_ => {
			error!("Disassembling non-concrete memory");
			return Err( () )	// Reading from non-concrete memory!
			}
		};

	match val
	{
	_ => {
		error!("Unknown opcode {:02x}", val);
		return Err( () )
		}
	}
}

// ---
// Helpers
// ---
fn sign_extend(bits: uint, value: u32) -> u64
{
	if( value >> (bits-1) != 0 ) {
		(value as u64) | ( ::std::u64::MAX << bits )
	}
	else {
		(value as u64)
	}
}

fn expand_imm_arm(imm12: u32) -> u64
{
	let val_ur = imm12 & 0xFF;
	let count = (((imm12 >> 8) & 0xF) * 2) as uint;
	((val_ur >> count) | (val_ur << (32 - count))) as u64
}

fn reg(word: u32, ofs: uint) -> InstrParam
{
	ParamTrueReg( ((word >> ofs) & 15) as u8 )
}

mod instrs
{
	use disasm::state::State;
	use disasm::{InstructionClass,InstrParam};

	// Set system register
	def_instr!(SET_SREG, InstrSetSReg, (f,p,state) => {
		{ false };
		{ write!(f, "SR{} {} {}", p[0], p[1], p[2]) };
		{
			let regid = match p[0] {
				::disasm::ParamImmediate(v) => v,
				_ => fail!("Invalid type for param[0] of SET_SREG, {}", p[0]),
				};
			let val = state.get(p[1]);
		};
		{
			unimplemented!();
		};
	})
	
	// Branch+Exchange
	def_instr!(BX, InstrBX, (f,p,state) => {
		{ true };
		{ write!(f, "{}", p[0]) };
		{
			unimplemented!();
		};
		{
			fail!("Can't reverse BX");
		};
	})
	
	// Branch+Link+Exchange
	def_instr!(BLX, InstrBLX, (f,p,state) => {
		{ true };
		{ write!(f, "{}", p[0]) };
		{
			unimplemented!();
		};
		{
			unimplemented!();
		};
	})
}

// vim: ft=rust
