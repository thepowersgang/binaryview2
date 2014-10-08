//
//
//
use value::Value;
use memory::MemoryStateAccess;

static NUM_TMPREGS: uint = 4;

/// Emulated CPU state during pseudo-execution
pub struct State<'mem>
{
	/// Reference to system memory
	memory: &'mem ::memory::MemoryState,
	/// Real registers
	registers: Vec<Value<u64>>,
	/// Temporary registers
	tmpregs: [Value<u64>,..NUM_TMPREGS],
	
	/// List of addresses to be processed on next pass
	todo_list: Vec<(u64, uint)>,
}

impl<'mem> State<'mem>
{
	/// Create a new empty state
	pub fn null<'a>(cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState) -> State<'a>
	{
		State {
			memory: mem,
			registers: Vec::from_fn(cpu.num_regs(), |_| Value::unknown()),
			tmpregs: [Value::unknown(), ..NUM_TMPREGS],
			todo_list: Vec::new(),
		}
	}

	/// Retrive the contents of the todo list
	pub fn todo_list(&self) -> &[(u64,uint)] {
		self.todo_list.as_slice()
	}
	
	/// Execute a single instruction
	pub fn run(&mut self, instr: &::disasm::Instruction)
	{
		instr.class.forwards(self, instr.params.as_slice());
	}
	
	
	/// Get the value of a parameter (register)
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
			Value::known(v)
			},
		}
	}
	/// Set the value of a parameter (register)
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
	
	/// Read from emulated memory
	pub fn read<T:Int+Unsigned+MemoryStateAccess+::std::fmt::LowerHex>(&mut self, addr: Value<u64>) -> Value<T>
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
	/// Write to emulated memory
	pub fn write<T:Int+Unsigned+MemoryStateAccess+::std::fmt::LowerHex>(&mut self, addr: Value<u64>, val: Value<T>)
	{
		debug!("write({} <= {})", addr, val);
	}

	/// Add an address to be processed	
	pub fn add_target(&mut self, val: Value<u64>, mode: uint)
	{
		debug!("add_target({}, mode={})", val, mode);
		if val.is_fixed_set()
		{
			for i in val.possibilities()
			{
				self.todo_list.push( (i,mode) );
			}
		}
	}
}

// vim: ft=rust
