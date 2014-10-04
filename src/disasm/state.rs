//
//
//
use value::Value;

static NUM_TMPREGS: uint = 4;

pub struct State<'mem>
{
	memory: &'mem ::memory::MemoryState,
	registers: Vec<Value<u64>>,
	tmpregs: [Value<u64>,..NUM_TMPREGS],
}

impl<'mem> State<'mem>
{
	pub fn null<'a>(cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState) -> State<'a>
	{
		State {
			memory: mem,
			registers: Vec::from_fn(cpu.num_regs(), |_| Value::unknown()),
			tmpregs: [Value::unknown(), ..NUM_TMPREGS],
		}
	}
	
	pub fn run(&self, instr: &::disasm::Instruction)
	{
		
	}
	
	pub fn set(&mut self, param: ::disasm::InstrParam, val: Value<u64>)
	{
		match param
		{
		::disasm::ParamTrueReg(r) => {
			assert!( (r as uint) < self.registers.len() );
			(*self.registers.get_mut(r as uint)) = val;
			},
		::disasm::ParamTmpReg(r) => {
			assert!( (r as uint) < NUM_TMPREGS );
			self.tmpregs[r as uint] = val;
			},
		::disasm::ParamImmediate(_) => fail!("Setting an immediate"),
		}
	}
}

// vim: ft=rust
