// BinaryView2
// - by John Hodge (thePowersGang)
//
// disam/cpus/arm.rs
// - Recent ARM CPU disassembly (written against ARMv5)
use value::{Value,ValueKnown};
use disasm::COND_ALWAYS;
use disasm::common_instrs;
use disasm::{Instruction,InstructionClass};
use disasm::{InstrParam,ParamImmediate,ParamTrueReg,ParamTmpReg};
use disasm::{InstrSizeNA,InstrSize8,InstrSize16,InstrSize32};

trait BitExtractor {
	fn bits(&self, base: uint, count: uint) -> Self;
}
impl BitExtractor for u16 {
	fn bits(&self, base: uint, count: uint) -> u16 {
		(*self >> base) & ((1 << count)-1)
	}
}

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

#[allow(non_snake_case)]
/// Disassemble code in ARM mode (32-bits per instruction)
fn disassemble_arm(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let word = try!(readmem::<u32>(mem, addr));

	let ccode = match (word >> 28) as u8
		{
		c @ 0x0 ... 0xD => c,
		0xE => COND_ALWAYS,
		0xF => {
			error!("TODO: Unconditional instructions");
			return Err( () );
			},
		v @ _ => fail!("Invalid (impossible) condition code in ARM {:x}", v),
		};
	
	let op = ((word>>20 & 0xFF) << 4) | (word>>4 & 0xf);
	Ok( match op
	{
        0x120 => {	// mov CPSR, Rn
		Instruction::new(
			4, ccode, InstrSize32,
			&instrs::SET_SREG,
			vec![
				ParamImmediate( SRegCPSR as u64 ),
				reg(word, 0),
				ParamImmediate( match (word>>20)&3 { 0=>0,1=>0,2=>0,_=>0 } ),
				]
			)
		},
	// Branch+Exchange Register
	0x121 => Instruction::new( 4, ccode, InstrSizeNA, &instrs::BX, vec![ reg(word, 0) ] ),
	// Branch+Link+Exchange
	0x123 => Instruction::new( 4, ccode, InstrSizeNA, &instrs::BLX, vec![ reg(word, 0) ] ),
	// Logical-Shift-Left (and Reg/Reg Move)
	0x1A0 => {
		let amt = (word >> 7) & 31;
		if amt == 0 {
			Instruction::new( 4, ccode, InstrSize32, &common_instrs::MOVE, vec![
				reg(word,12), reg(word,0)
				] )
		}
		else {
			Instruction::new( 4, ccode, InstrSize32, &common_instrs::SHL, vec![
				reg(word,12), reg(word,0), ParamImmediate(amt as u64)
				] )
		}
		},
	// Logical-Shift-Left
	0x1A1 => Instruction::new( 4, ccode, InstrSize32, &common_instrs::SHL, vec![
			reg(word,12), reg(word,0), reg(word,8)
			]
		),
	// Add (Register, Immediate)
	0x280 ... 0x28F => Instruction::new(
		4, ccode, InstrSize32, &common_instrs::ADD,
		vec![
			reg(word, 12), reg(word, 16), ParamImmediate( expand_imm_arm(word & 0xFFF) ),
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
			4, ccode, InstrSize32,
			&common_instrs::MOVE,
			vec![
				ParamTrueReg( Rd ),
				ParamImmediate( expand_imm_arm(word & 0xFFF) ),
				]
			)
		},
	// STR Rd, [Rn,#imm12]
	0x580 ... 0x58F => Instruction::new(
		4, ccode, InstrSize32,
		&common_instrs::STORE_OFS,
		vec![
			ParamTrueReg( ((word>>12)&0xF) as u8 ),
			ParamTrueReg( ((word>>16)&0xF) as u8 ),
			ParamImmediate( sign_extend(12, word & 0xFFF) ),
			]
		),
	// LDR Rd, [Rn,#imm12]
	0x590 ... 0x59F => Instruction::new(
		4, ccode, InstrSize32,
		&common_instrs::LOAD_OFS,
		vec![
			ParamTrueReg( ((word>>12)&0xF) as u8 ),
			ParamTrueReg( ((word>>16)&0xF) as u8 ),
			ParamImmediate( sign_extend(12, word & 0xFFF) ),
			]
		),
	0xA00 ... 0xAFF => {
		// Jump to Address+opr*4+8
		Instruction::new(
			4, ccode, InstrSizeNA,
			&common_instrs::JUMP as &InstructionClass,
			vec![ ParamImmediate(addr + 8 + sign_extend(24, word & 0xFFFFFF) * 4), ]
			)
		},
	0xB00 ... 0xBFF => {
		// Branch+Link (call) Address+opr*4+8
		Instruction::new(
			4, ccode, InstrSizeNA,
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

#[allow(non_snake_case)]	// Suppresses warning on Rd/Rn/Rt
/// Disassemble in THUMB mode
fn disassemble_thumb(mem: &::memory::MemoryState, addr: u64) -> Result<::disasm::Instruction,()>
{
	let word = try!(readmem::<u16>(mem, addr));

	Ok(match word >> 10
	{
	// Logical Shift Left
	0x00 ... 0x01 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::SHL,
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( word.bits(6,5) as u64) ]
		),
	// Logical Shift Right
	0x02 ... 0x03 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::SHR,
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( word.bits(6,5) as u64) ]
		),
	// Arithmetic Shift Right
	0x04 ... 0x05 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &instrs::ASR,
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( word.bits(6,5) as u64) ]
		),
	// Add/Sub reg
	0x06 => Instruction::new(
		2, COND_ALWAYS, InstrSize32,
		if (word >> 9) & 1 != 0 { &common_instrs::SUB } else { &common_instrs::ADD },
		vec![ reg_t(word, 0), reg_t(word, 3), reg_t(word, 6) ]
		),
	// Add/Sub imm3
	0x07 => Instruction::new(
		2, COND_ALWAYS, InstrSize32,
		if (word >> 9) & 1 != 0 { &common_instrs::SUB } else { &common_instrs::ADD },
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( ((word >> 6) & 7) as u64 ) ]
		),
	// MOV Rd, #imm8
	0x08 ... 0x09 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::MOVE,
		vec![ reg_t(word, 8), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// CMP Rd, #imm8
	0x0a ... 0x0b => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,	// < Use SUB and assign to #tr0
		vec![ ParamTmpReg(0), reg_t(word, 8), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// ADD Rd, Rd, #imm8
	0x0c ... 0x0d => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
		vec![ reg_t(8, 3), reg_t(8, 3), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// SUB Rd, Rd, #imm8
	0x0e ... 0x0f => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,
		vec![ reg_t(8, 3), reg_t(8, 3), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// 0x10 - Data Processing (Sect A6-8)
	0x10 => match (word >> 6) & 0xF
		{
		v @ 0x0 ... 0x07 | v @ 0xc ... 0xe => Instruction::new(
				2, COND_ALWAYS, InstrSize32,
				match v
				{
				0x0 => &common_instrs::AND as &InstructionClass,
				0x1 => &common_instrs::XOR as &InstructionClass,
				0x2 => &common_instrs::SHL as &InstructionClass,
				0x3 => &common_instrs::SHR as &InstructionClass,
				0x4 => &instrs::ASR        as &InstructionClass,
				0x5 => &common_instrs::ADD as &InstructionClass,
				0x6 => &common_instrs::SUB as &InstructionClass,
				0x7 => &common_instrs::ROR as &InstructionClass,
				0xc => &common_instrs::OR  as &InstructionClass,
				0xd => &common_instrs::MUL as &InstructionClass,
				0xe => &instrs::BIC        as &InstructionClass,
				_ => fail!("ARM THUMB 0x10:{{0-7,c-e}} Unmatched {}", v)
				},
				vec![ reg_t(word, 0), reg_t(word,3), reg_t(word,6) ]
			),
		// - TEST Rt, Rn
		0x8 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::AND,
			vec![ ParamTmpReg(0), reg_t(word, 0), reg_t(word,3) ]
			),
		// - Reverse Subtract (RSB) (Negate?)
		0x9 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,
			vec![ reg_t(word, 0), ParamImmediate(0), reg_t(word,3) ]
			),
		// - CMP Rt, Rn
		0xA => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,
			vec![ ParamTmpReg(0), reg_t(word, 0), reg_t(word,3) ]
			),
		0xB => {
			error!("ARM THUMB 0x10:B Undefined");
			return Err( () );
			},
		// - NOT
		0xF => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::NOT,
			vec![ reg_t(word, 0), reg_t(word,3) ]
			),
		v @ _ => fail!("ARM THUMB 0x10 Unmatched {:x}", v)
		},
	// 0x11: Special data instructions, branch and exchange
	0x11 => {
		let Rd = (word.bits(0,3) | word.bits(7,1) << 3) as u8;
		match (word >> 6) & 0xF
		{
		// ADD R, R, R
		0x0 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
			vec![ reg_t(word, 0), reg_t(word, 3), reg_t(word, 6) ]
			),
		// ADD Rd, Rd, Rn (high)
		0x1 ... 0x3 => {
			if Rd == 15 {
				error!("TODO: MOV PC, PC+Rn");
				return Err( () );
			}
			else {
				Instruction::new(
					2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
					vec![ ParamTrueReg(Rd), ParamTrueReg(Rd), reg(word as u32,3) ]
					)
			}
			},
		0x4 => {
			error!("UNPREDICTABLE Thumb 0x11:4");
			return Err( () );
			},
		0x5 ... 0x7 => Instruction::new(
				2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,
				vec![ ParamTmpReg(0), ParamTrueReg(Rd), reg(word as u32, 3) ]
			),
		// Move Register (High)
		0x9 ... 0xb => {
			let Rn = (word >> 3) & 0xF;
			if Rd == 15 {
				Instruction::new(2, COND_ALWAYS, InstrSizeNA, &common_instrs::JUMP,
					vec![ ParamTrueReg(Rn as u8) ])
			}
			else {
				Instruction::new(2, COND_ALWAYS, InstrSize32, &common_instrs::MOVE,
					vec![ ParamTrueReg(Rd as u8), ParamTrueReg(Rn as u8) ])
			}
			},
		// BX Rd
		0xc ... 0xd => Instruction::new(
			2, COND_ALWAYS, InstrSizeNA, &common_instrs::JUMP,
			vec![ reg(word as u32, 3) ]
			),
		v @ _ => {
			error!("Unknown opcode 11:{:x}", v);
			return Err( () )
			},
		}
		},
	// LDR Rt, [PC,#imm8]
	0x12 ... 0x13 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::LOAD_OFS,
		vec![ reg_t(word, 0), ParamImmediate((addr + 4) & !3), ParamImmediate( (word.bits(0,8)*4) as u64 ) ]
		),
	// (STR|LDR) Rt, [Rn,#imm5]
	0x18 ... 0x1B => Instruction::new(
		2, COND_ALWAYS, InstrSize32,
		if word.bits(11,1) != 0 { &common_instrs::LOAD_OFS } else { &common_instrs::STORE_OFS },
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( (word.bits(6,5) * 4) as u64 ) ]
		),
	// (STR|LDR)B Rt, [Rn,#imm5]
	0x1C ... 0x1F => Instruction::new(
		2, COND_ALWAYS, InstrSize8,
		if word.bits(11,1) != 0 { &common_instrs::LOAD_OFS } else { &common_instrs::STORE_OFS },
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( (word.bits(6,5) * 1) as u64 ) ]
		),
	// (STR|LDR)H Rt, [Rn,#imm5]
	0x20 ... 0x23 => Instruction::new(
		2, COND_ALWAYS, InstrSize16,
		if word.bits(11,1) != 0 { &common_instrs::LOAD_OFS } else { &common_instrs::STORE_OFS },
		vec![ reg_t(word, 0), reg_t(word, 3), ParamImmediate( (word.bits(6,5) * 2) as u64 ) ]
		),
	// (STR|LDR) Rt, [SP,#imm8]
	0x24 ... 0x27 => Instruction::new(
		2, COND_ALWAYS, InstrSize32,
		if word.bits(11,1) != 0 { &common_instrs::LOAD_OFS } else { &common_instrs::STORE_OFS },
		vec![ reg_t(word, 8), ParamTrueReg(13), ParamImmediate( (word.bits(6,8) * 4) as u64 ) ]
		),
	// ADR Rd, [PC,#imm8]
	0x28 ... 0x29 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
		vec![ reg_t(word, 8), ParamTrueReg(15), ParamImmediate( (word.bits(6,8) * 4) as u64 ) ]
		),
	// ADD Rd, SP, #imm8*4
	0x2A ... 0x2B => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
		vec![ reg_t(word, 8), ParamTrueReg(13), ParamImmediate( (word.bits(6,8) * 4) as u64 ) ]
		),
	// Misc Instructions (A6..2.5)
	0x2C => match (word >> 5) & 0x1F
		{
		// ADD SP, SP, #imm5
		0x0 ... 0x3 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::ADD,
			vec![ ParamTrueReg(13), ParamTrueReg(13), ParamImmediate( (word.bits(0,5) * 4) as u64 ) ]
			),
		// SUB SP, SP, #imm5
		0x4 ... 0x7 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &common_instrs::SUB,
			vec![ ParamTrueReg(13), ParamTrueReg(13), ParamImmediate( (word.bits(0,5) * 4) as u64 ) ]
			),
		v @ _ => {
			error!("Unknown opcode 2C:{:x}", v);
			return Err( () );
			},
		},
	// Misc Instructions
	0x2D => match (word >> 5) & 0x1F
		{
		0x0 ... 0xF => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &instrs::PUSH_M,
			// Bitmask. Instr[8] = LR
			vec![ ParamImmediate( (((word >> 8) & 1) << 14 | (word & 0xFF)) as u64) ]
			),
		v @ _ => {
			error!("Unknown opcode 2D:{:x}", v);
			return Err( () );
			},
		},
	0x2F => match word.bits(8, 2)
		{
		// POP Multiple
		0x0 ... 0x1 => Instruction::new(
			2, COND_ALWAYS, InstrSize32, &instrs::POP_M,
			// Bitmask. Instr[8] = PC
			vec![ ParamImmediate( (((word >> 8) & 1) << 15 | (word & 0xFF)) as u64) ]
			),
		v @ _ => {
			error!("Unknown opcode 2F{:x}", v);
			return Err( () );
			},
		},
	// STM - Store Multiple
	0x30 ... 0x31 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &instrs::STM,
		vec![ reg_t(word, 8), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// LDM - Load Multiple
	0x32 ... 0x33 => Instruction::new(
		2, COND_ALWAYS, InstrSize32, &instrs::LDM,
		vec![ reg_t(word, 8), ParamImmediate( (word & 0xFF) as u64 ) ]
		),
	// Conditional Branch + Supervisor Call
	0x34 ... 0x37 => match word.bits(8, 4)
		{
		0x0 ... 0xD => Instruction::new(
			2, word.bits(8,4) as u8, InstrSizeNA, &common_instrs::JUMP,
			vec![ ParamImmediate(addr + 4 + sign_extend(9, (word.bits(0,8)*2) as u32)) ]
			),
		0xE => return Err( () ),
		0xF => Instruction::new(
			2, COND_ALWAYS, InstrSizeNA, &instrs::SVC,
			vec![ ParamImmediate(word.bits(0, 8) as u64) ]
			),
		_ => fail!(""),
		},
	// B imm11
	0x38 ... 0x39 => Instruction::new(
		2, COND_ALWAYS, InstrSizeNA, &common_instrs::JUMP,
		vec![ ParamImmediate(addr + 4 + sign_extend(12, (word.bits(0,11)*2) as u32)) ]
		),
	// 32-bit instructions
	0x3a ... 0x3f => {
		let word2 = try!(readmem::<u16>(mem, addr+2));
		match (word >> 11) & 3
		{
		0 => {
			error!("Thumb 3F:0 Undefined");
			return Err( () )
			},
		1 => {
			if (word >> 10) & 1 != 0
			{
				// Coprocessor
				error!("TODO: Thumb 3F:1 Coprocessor");
				return Err( () );
			}
			else if (word >> 9) & 1 != 0
			{
				// Data Processing (Shifted Reg)
				error!("TODO: Thumb 3F:1 DPSR");
				return Err( () );
			}
			else if (word >> 6) & 1 != 0
			{
				// Load/store dual, load/store excl, table branch
				error!("TODO: Thumb 3F:1 Load/Store Dual/Excl, Table Branch");
				return Err( () );
			}
			else
			{
				// Load/Store Multiple
				error!("TODO: Thumb 3F:1 Load/Store Multiple");
				return Err( () );
			}
			},
		2 => {
			if (word >> 15) & 1 != 0
			{
				match (word2 >> 12) & 7
				{
				4 ... 7 => { // BL/BLX
					let flag = (word >> 10) & 1 != 0;
					// TODO: Check logic of this snippet... I don't trust it
					let ofs = ((word2 as u32 & 0x7FF) << 1)
						| (word as u32 & 0x3FF) << 12
						| if flag {
							((word2 as u32 >> 11) & 1) << 22
							| ((word2 as u32 >> 13) & 1) << 23
							| 1 << 24
							} else {
							0
							}
						;
					
					if (word2>>12) & 1 == 0 {
						// Switch to ARM mode
						Instruction::new(4, COND_ALWAYS, InstrSizeNA, &instrs::BLX,
							vec![ ParamImmediate( addr + 4 + sign_extend(25, ofs) ) ])
					}
					else {
						Instruction::new(4, COND_ALWAYS, InstrSizeNA, &common_instrs::CALL,
							vec![ ParamImmediate( addr + 4 + sign_extend(25, ofs) ) ])
					}
					},
				v @ _ => {
					error!("Unknown 3F:2 Branch/Misc {:x}", v);
					return Err( () );
					},
				}
			}
			else if (word >> 9) & 1 != 0
			{
				match (word >> 4) & 31
				{
				v @ _ => {
					error!("Unknown 3F:2 Data Binary Imm{:x}", v);
					return Err( () );
					},
				}
			}
			else
			{
				match (word >> 4) & 31
				{
				v @ _ => {
					error!("Unknown 3F:2 Data Mod Imm{:x}", v);
					return Err( () );
					},
				}
			}
			},
		3 => {
			if (word >> 10) & 1 != 0
			{
				error!("TODO: Thumb Coprocessor (3[A-F]:3)");
				return Err( () );
			}
			else
			{
				error!("Unknown thumb 32-bit instr (3[A-F]:3) {:04x} {:04x}", word, word2);
				return Err( () );
			}
			},
		_ => fail!("impossible (thumb 3F)"),
		}
		},
	v @ _ => {
		error!("Unknown opcode {:02x}", v);
		return Err( () )
		}
	})
}

fn readmem<T: ::value::ValueType+::memory::MemoryStateAccess>(mem: &::memory::MemoryState, addr: u64) -> Result<T,()>
{
	use memory::MemoryStateAccess;
	match MemoryStateAccess::read(mem, addr)
	{
	Some(ValueKnown(x)) => Ok(x),
	Some(_) => {
		error!("Disassembling non-concrete memory at {:#x}", addr);
		Err( () )
		},
	None => {
		error!("Disassembling unmapped memory at {:#x}", addr);
		Err( () )
		},
	}
}

// ---
// Helpers
// ---
fn sign_extend(bits: uint, value: u32) -> u64
{
	if value >> (bits-1) != 0 {
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
fn reg_t(word: u16, ofs: uint) -> InstrParam
{
	ParamTrueReg( ((word >> ofs) & 7) as u8 )
}

mod instrs
{
	use value::Value;
	use disasm::state::State;
	use disasm::{InstrParam};

	// Set system register
	def_instr!(SET_SREG, InstrSetSReg, (f,instr,p,state) => {
		{ false };
		{ write!(f, "SR{} {} {}", p[0], p[1], p[2]) };
		{
			let regid = match p[0] {
				::disasm::ParamImmediate(v) => v,
				_ => fail!("Invalid type for param[0] of SET_SREG, {}", p[0]),
				};
			let val = state.get(p[1]);
			warn!("TODO: Assign SReg {} value {}", regid, val);
		};
		{
			unimplemented!();
		};
	})
	
	def_instr!(SVC, InstrSVC, (f,instr,p,state) => {
		{ false };
		{ write!(f, "{}", p[0]) };
		{
			warn!("TODO: ARM SVC - Apply based on some description (Arg={})", p[0]);
			state.clobber_everything();
		};
		{
			unimplemented!();
		};
	})
	
	// Branch+Exchange
	def_instr!(BX, InstrBX, (f,instr,p,state) => {
		{ true };
		{ write!(f, "{}", p[0]) };
		{
			let addr = state.get(p[0]);
			let mode = addr & Value::known(1);
			if mode == Value::known(1)
			{
				state.add_target(addr & Value::known(!1), 1)
			}
			else if mode == Value::known(0)
			{
				state.add_target(addr & Value::known(!1), 0)
			}
			else
			{
				// No idea! Can't add target
			}
		};
		{
			let _ = p; let _ = state;
			fail!("Can't reverse BX");
		};
	})
	
	// Branch+Link+Exchange
	def_instr!(BLX, InstrBLX, (f,instr,p,state) => {
		{ true };
		{ write!(f, "{}", p[0]) };
		{
			let addr = state.get(p[0]);
			error!("TODO: BLX {}", addr);
			unimplemented!();
		};
		{
			let _ = p;
			let _ = state;
			unimplemented!();
		};
	})
	
	// Push multiple (bitmask)
	def_instr!(PUSH_M, InstrPushMulti, (f,instr,p,state) => {
		{ false };
		{
			let mask = p[0].immediate();
			for i in range(0,16) {
				if mask & 1 << i != 0 {
					try!(write!(f, "R{} ", i));
				}
			}
			Ok( () )
		};
		{
			let mask = p[0].immediate();
			debug!("mask={:x}", mask);
			for i in range(0,16).rev() {
				if mask & 1 << i != 0 {
					// TODO: Decrement stack pointer
					let val = state.get( ::disasm::ParamTrueReg(i as u8) );
					state.stack_push( val );
				}
			}
		};
		{
			let _ = p;
			let _ = state;
			unimplemented!();
		};
	})

	// Pop multiple (bitmask)
	def_instr!(POP_M, InstrPopMulti, (f,instr,p,state) => {
		{
			let mask = p[0].immediate();
			mask & (1 << 15) != 0
		};
		{
			let mask = p[0].immediate();
			for i in range(0,16) {
				if mask & 1 << i != 0 {
					try!(write!(f, "R{} ", i));
				}
			}
			Ok( () )
		};
		{
			let mask = p[0].immediate();
			debug!("mask={:x}", mask);
			for i in range(0,16) {
				if mask & 1 << i != 0 {
					// TODO: Decrement stack pointer
					let val = state.stack_pop();
					state.set( ::disasm::ParamTrueReg(i as u8), val );
				}
			}
		};
		{
			let _ = p;
			let _ = state;
			unimplemented!();
		};
	})
	// Store multiple (bitmask)
	def_instr!(STM, InstrSTM, (f,instr,p,state) => {
		{ false };
		{
			try!( write!(f, "{}", p[0]) );
			let mask = p[1].immediate();
			for i in range(0,16) {
				if mask & 1 << i != 0 {
					try!(write!(f, "R{} ", i));
				}
			}
			Ok( () )
		};
		{
			let mut addr = state.get(p[0]);
			let mask = p[1].immediate();
			debug!("mask={:x}", mask);
			for i in range(0,16).rev() {
				if mask & 1 << i != 0 {
					let val = state.get( ::disasm::ParamTrueReg(i as u8) );
					state.write(addr, val);
					// TODO: Support alternate types of STM
					addr = addr + Value::known(4);
				}
			}
			state.set(p[0], addr);
		};
		{
			let _ = p;
			let _ = state;
			unimplemented!();
		};
	})
	// Store multiple (bitmask)
	def_instr!(LDM, InstrLDM, (f,instr,p,state) => {
		{ false };
		{
			try!( write!(f, "{}", p[0]) );
			let mask = p[1].immediate();
			for i in range(0,16) {
				if mask & 1 << i != 0 {
					try!(write!(f, "R{} ", i));
				}
			}
			Ok( () )
		};
		{
			let mut addr = state.get(p[0]);
			let mask = p[1].immediate();
			debug!("mask={:x}", mask);
			for i in range(0,16).rev() {
				if mask & 1 << i != 0 {
					let val = state.read(addr);
					state.set( ::disasm::ParamTrueReg(i as u8), val );
					// TODO: Support alternate types of LDM
					addr = addr + Value::known(4);
				}
			}
			state.set(p[0], addr);
		};
		{
			let _ = p;
			let _ = state;
			unimplemented!();
		};
	})

	// ASR - Arithmetic Shift Right
	def_instr!(ASR, IClassAsr, (f, instr, params, state) => {
		{ false };
		{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
		{
			let v = state.get(params[1]);
			let count = state.get(params[2]);
			if let Some(c) = count.val_known()
			{
				let base_mask = match v.bit(v.bitsize()-1)
					{
					::value::ValueBoolUnknown => Value::unknown(),
					::value::ValueBoolTrue => Value::ones(),
					::value::ValueBoolFalse => Value::zero(),
					};
				if c >= v.bitsize() as u64 {
					warn!("Overshift in ASR {} >= {}", c, v.bitsize());
					state.set(params[0], base_mask);
				}
				else {
					let c = c as uint;
					let (_,mask) = base_mask << c;
					let (_out,base_val) = v >> c;
					let val = base_val | mask;
					state.set(params[0], val);
					//state.set_flag(FlagCarry, extra & Value::known(1))
				}
			}
			else
			{
				warn!("TODO: ASR by a set/range of values");
				state.set(params[0], Value::unknown());
			}
		};
		{ unimplemented!(); };
	})

	// BIC - Bit Clear
	// AND with NOT of provided mask
	def_instr!(BIC, IClassBic, (f, instr, params, state) => {
		{ false };
		{ write!(f, "{}, {}, {}", params[0], params[1], params[2]) };
		{
			let v = state.get(params[1]);
			let mask = state.get(params[2]);
			let val = v & !mask;
			state.set(params[0], val);
		};
		{ unimplemented!(); };
	})
}

// vim: ft=rust
