//
//
//
use value::Value;

static NUM_TMPREGS: uint = 4;

pub struct State<'mem>
{
	cpu: &'mem ::disasm::CPU + 'mem,
	memory: &'mem ::memory::MemoryState,
	registers: Vec<Value<u64>>,
	tmpregs: [Value<u64>,..NUM_TMPREGS],
}

impl<'mem> State<'mem>
{
	pub fn null<'a>(cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState) -> State<'a>
	{
		State {
			cpu: cpu,
			memory: mem,
			registers: Vec::from_fn(cpu.num_regs(), |_| Value::unknown()),
			tmpregs: [Value::unknown(), ..NUM_TMPREGS],
		}
	}
	
	pub fn run(&self, instr: &::disasm::Instruction)
	{
		
	}
}

// vim: ft=rust
