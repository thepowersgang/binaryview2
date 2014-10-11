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
		let v = match param
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
			};
		debug!("get({}) = {}", param, v);
		v
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
		// TODO: Tag unknown values such that accesses to an unknown base can be tracked
		// > Tag with origin of unknown? Probably
		// > Tagging will allow types of object fields to be tracked
		let ret = if let Some(addr_val) = addr.val_known()
			{
				match MemoryStateAccess::read(self.memory, addr_val)
				{
				Some(x) => x,
				None => {
					warn!("Reading unmapped memory {}", addr_val);
					Value::unknown()
					}
				}
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
		error!("TODO: Support write access to simulated memory");
		// Requirements:
		// - Store locally a set of changes applied by this state
		//  > Read should query this first.
		// - This list is accessed by disasm code and applied to main memory as a value set once state is destroyed
	}

	pub fn stack_push(&mut self, val: Value<u64>)
	{
		error!("TODO: State push value ({})", val); 
	}
	pub fn stack_pop(&mut self) -> Value<u64>
	{
		error!("TODO: state pop value");
		Value::unknown()
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
