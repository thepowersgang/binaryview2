//
//
//
use value::Value;
use memory::MemoryStateAccess;

static NUM_TMPREGS: uint = 4;

pub struct State<'mem>
{
	memory: &'mem ::memory::MemoryState,
	registers: Vec<Value<u64>>,
	tmpregs: [Value<u64>,..NUM_TMPREGS],
	
	todo_list: Vec<(u64, uint)>,
}

impl<'mem> State<'mem>
{
	pub fn null<'a>(cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState) -> State<'a>
	{
		State {
			memory: mem,
			registers: Vec::from_fn(cpu.num_regs(), |_| Value::unknown()),
			tmpregs: [Value::unknown(), ..NUM_TMPREGS],
			todo_list: Vec::new(),
		}
	}

	pub fn todo_list(&self) -> &[(u64,uint)] {
		self.todo_list.as_slice()
	}
	pub fn run(&mut self, instr: &::disasm::Instruction)
	{
		instr.class.forwards(self, instr.params.as_slice());
	}
	
	pub fn get(&mut self, param: ::disasm::InstrParam) -> Value<u64>
	{
		match param
		{
		::disasm::ParamTrueReg(r) => {
			assert!( (r as uint) < self.registers.len() );
			self.registers[r as uint]
			},
		::disasm::ParamTmpReg(r) => {
			assert!( (r as uint) < NUM_TMPREGS );
			self.tmpregs[r as uint]
			},
		::disasm::ParamImmediate(v) => {
			Value::fixed(v)
			},
		}
	}
	pub fn set(&mut self, param: ::disasm::InstrParam, val: Value<u64>)
	{
		debug!("set({} = {})", param, val);
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
	
	pub fn read<T:Int+MemoryStateAccess+::std::fmt::LowerHex>(&mut self, addr: Value<u64>) -> Value<T>
	{
		let ret = if let Some(addr_val) = addr.val_known()
			{
				MemoryStateAccess::read(self.memory, addr_val)
			}
			else if addr.is_fixed_set()
			{
				fail!("TODO: Support generating set of data from read");
				Value::<T>::unknown()
			}
			else
			{
				// Unknown address = unknown data
				Value::<T>::unknown()
			};
		debug!("read({}) = {}", addr, ret);
		ret
	}
	pub fn write<T:Int+::std::fmt::LowerHex>(&mut self, addr: Value<u64>, val: Value<T>)
	{
		debug!("write({} <= {})", addr, val);
	}
	
	pub fn add_target(&mut self, val: Value<u64>)
	{
		if val.is_fixed_set()
		{
			for i in val.possibilities()
			{
				self.todo_list.push( (i,0) );
			}
		}
	}
}

// vim: ft=rust
