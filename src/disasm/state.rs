//
//
//
use value::{Value,ValueBool,ValueType};
use memory::MemoryStateAccess;
use disasm::instruction::InstrParam;
use disasm::CodePtr;
use std::collections::BitvSet;
use std::default::Default;

const NUM_TMPREGS: usize = 4;

/// Emulated CPU state during pseudo-execution
pub struct State<'mem>
{
	/// Execution/Simulation mode
	mode: RunMode,
	
	/// Reference to system memory
	memory: &'mem ::memory::MemoryState,
	
	/// List of addresses to be processed on next pass
	todo_list: Vec<(CodePtr, bool)>,

	/// State data (flags, registers)
	data: StateData,
}

pub enum RunMode
{
	/// Minimal state propagation
	Parse,
	/// Stack enabled
	Blockify,
	/// Stack and parital memory?
	CallingConv,
	/// Full memory and stack works
	Full,
}

/// State data (stored separately to allow saving)
pub struct StateData
{
	/// Markers indicating that the specified register was read before being written
	inputs: BitvSet,
	/// Helper for maintaining `inputs`
	writtens: BitvSet,
	
	/// Real registers - Static vector
	registers: Vec<Value<u64>>,
	/// Temporary registers
	tmpregs: [Value<u64>; NUM_TMPREGS],
	
	/// Stack - Dynamic vector
	stack: Vec<Value<u64>>,
	
	/// Carry flag
	flag_c: ValueBool,
	/// Overflow flag
	flag_v: ValueBool,
}

pub enum StatusFlags
{
	Carry,
	Overflow,
}

impl<'mem> State<'mem>
{
	/// Create a new empty state
	pub fn null<'a>(mode: RunMode, cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState) -> State<'a>
	{
		State {
			mode: mode,	// TODO: Receive as an argument
			memory: mem,
			data: StateData::new(cpu),
			todo_list: Vec::new(),	
		}
	}
	pub fn from_data<'a>(mode: RunMode, cpu: &'a ::disasm::CPU, mem: &'a ::memory::MemoryState, data: StateData) -> State<'a>
	{
		State {
			mode: mode,
			memory: mem,
			data: data,
			todo_list: Vec::new(),
		}
	}
	
	//pub fn fill_canary(&mut self)
	//{
	//	for reg in self.data.registers.iter_mut()
	//	{
	//		*reg = Value::canary();
	//	}
	//}
	
	//pub fn data(&self) -> &StateData {
	//	&self.data
	//}
	pub fn unwrap_data(self) -> StateData {
		self.data
	}

	/// Retrive the contents of the todo list
	pub fn todo_list(&self) -> &[(CodePtr,bool)] {
		self.todo_list.as_slice()
	}
	pub fn clear_todo_list(&mut self) {
		self.todo_list.clear()
	}
	
	/// Execute a single instruction
	pub fn run(&mut self, instr: &::disasm::instruction::Instruction)
	{
		debug!("--- {}", instr);
		instr.class.forwards(self, instr);
	}
	
	/// Get the value of a parameter (register)
	pub fn get(&mut self, param: InstrParam) -> Value<u64>
	{
		let v = match param
			{
			InstrParam::TrueReg(r)   => self.data.read_reg(r),
			InstrParam::TmpReg(r)    => self.data.read_tmp(r),
			InstrParam::Immediate(v) => Value::known(v),
			};
		debug!("get({:?}) = {:?}", param, v);
		v
	}
	/// Set the value of a parameter (register)
	pub fn set(&mut self, param: InstrParam, val: Value<u64>)
	{
		debug!("set({:?} = {:?})", param, val);
		match param
		{
		InstrParam::TrueReg(r) => self.data.write_reg(r, val),
		InstrParam::TmpReg(r)  => self.data.write_tmp(r, val),
		InstrParam::Immediate(_) => panic!("Setting an immediate"),
		}
	}
	
	/// Read from emulated memory
	pub fn read<T:ValueType+MemoryStateAccess>(&mut self, addr: &Value<u64>) -> Value<T>
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
				panic!("TODO: Support generating set of data from read");
				//Value::<T>::unknown()
			}
			else
			{
				// Unknown address = unknown data
				Value::<T>::unknown()
			};
		debug!("read({:?}) = {:?}", addr, ret);
		ret
	}
	/// Write to emulated memory
	pub fn write<T:ValueType+MemoryStateAccess>(&mut self, addr: &Value<u64>, val: Value<T>)
	{
		debug!("write({:?} <= {:?})", addr, val);
		match self.mode
		{
		RunMode::Full => {
			error!("TODO: Support write access to simulated memory");
			// Requirements:
			// - Store locally a set of changes applied by this state
			//  > Read should query this first.
			// - This list is accessed by disasm code and applied to main
			//   memory as a value set once state is destroyed.
			},
		_ => {},
		}
	}

	pub fn flag_set(&mut self, flag: StatusFlags, val: ValueBool)
	{
		match flag
		{
		StatusFlags::Carry    => { self.data.flag_c = val; },
		StatusFlags::Overflow => { self.data.flag_v = val; },
		}
	}
	pub fn flag_get(&self, flag: StatusFlags) -> ValueBool
	{
		match flag
		{
		StatusFlags::Carry    => self.data.flag_c,
		StatusFlags::Overflow => self.data.flag_v,
		}
	}
	
	pub fn stack_push(&mut self, val: Value<u64>)
	{
		debug!("stack_push({:?})", val);
		match self.mode
		{
		RunMode::Blockify|RunMode::Full => {
			self.data.stack.push(val);
			},
		_ => {},
		}
	}
	pub fn stack_pop(&mut self) -> Value<u64>
	{
		let rv = match self.mode
			{
			RunMode::Blockify|RunMode::Full =>
				match self.data.stack.pop()
				{
				Some(x) => x,
				None => {
					error!("Pop from empty stack");
					Value::unknown()
					},
				},
			_ => Value::unknown(),
			};
		debug!("stack_pop() = {:?}", rv);
		rv
	}

	/// Add an address to be processed	
	pub fn jump(&mut self, val: Value<u64>, mode: super::CPUMode)
	{
		debug!("jump({:?}, mode={})", val, mode);
		if val.is_fixed_set()
		{
			for addr in val.possibilities()
			{
				self.todo_list.push( (CodePtr::new(mode, addr),false) );
			}
		}
	}
	
	pub fn call(&mut self, val: Value<u64>, mode: super::CPUMode)
	{
		if val.is_fixed_set()
		{
			for addr in val.possibilities()
			{
				self.todo_list.push( (CodePtr::new(mode, addr),true) );
			}
			warn!("TODO: Call clobbering {:?} mode={}", val, mode);
			//if let Some(f) = state.functions.find( (mode,val
			//{
			//}
			//else
			//{
				// Fallback, clobber everything!
				self.clobber_everything();
			//}
		}
		else
		{
			// Fallback, clobber everything!
			self.clobber_everything();
		}
	}
	
	/// Clobber every register
	pub fn clobber_everything(&mut self)
	{
		for r in self.data.registers.iter_mut()
		{
			*r = Value::unknown();
		}
	}
}

impl StateData
{
	fn new(cpu: &::disasm::CPU) -> StateData
	{
		StateData {
			registers: (0 .. cpu.num_regs()).map(|_| Value::unknown()).collect(),
			stack: Vec::with_capacity(16),
			.. ::std::default::Default::default()
		}
	}
	
	fn read_reg(&mut self, idx: u8) -> Value<u64>
	{
		assert!( (idx as usize) < self.registers.len(), "Register index out of range");
		if ! self.writtens.contains(&(idx as usize))
		{
			self.inputs.insert( idx as usize );
		}
		self.registers[idx as usize].clone()
	}
	fn read_tmp(&self, idx: u8) -> Value<u64>
	{
		assert!( (idx as usize) < NUM_TMPREGS, "Temp register index out of range" );
		self.tmpregs[idx as usize].clone()
	}
	fn write_reg(&mut self, idx: u8, val: Value<u64>)
	{
		assert!( (idx as usize) < self.registers.len(), "Register index out of range");
		self.writtens.insert( idx as usize );
		self.registers[idx as usize] = val
	}
	fn write_tmp(&mut self, idx: u8, val: Value<u64>)
	{
		assert!( (idx as usize) < NUM_TMPREGS, "Temp register index out of range" );
		self.tmpregs[idx as usize] = val
	}
}

impl ::std::default::Default for StateData
{
	fn default() -> StateData
	{
		StateData {
			inputs: Default::default(),
			writtens: Default::default(),
			
			registers: Vec::new(),
			tmpregs: [Value::unknown(), Value::unknown(), Value::unknown(), Value::unknown()],
			stack: Vec::new(),
			
			flag_c: ValueBool::Unknown,
			flag_v: ValueBool::Unknown,
		}
	}
}

// Note: Can't derive becuase fixed-size arrays clone using Copy
impl ::std::clone::Clone for StateData
{
	fn clone(&self) -> StateData
	{
		StateData {
			inputs: self.inputs.clone(),
			writtens: self.writtens.clone(),
			
			registers: self.registers.clone(),
			tmpregs: [self.tmpregs[0].clone(), self.tmpregs[1].clone(), self.tmpregs[2].clone(), self.tmpregs[3].clone()],
			stack: self.stack.clone(),
			
			flag_c: self.flag_c.clone(),
			flag_v: self.flag_v.clone(),
		}
	}
}

impl ::std::fmt::Debug for StateData
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		try!( write!(f, "StateData {{\n") );
		for (i,reg) in self.registers.iter().enumerate()
		{
			try!( write!(f, "  R{}={:?}", i, reg) );
		}
		try!( write!(f, "\n") );
		for (i,reg) in self.tmpregs.iter().enumerate()
		{
			try!( write!(f, "  tr#{}={:?}", i, reg) );
		}
		try!( write!(f, "\n") );
		try!( write!(f, "  Stack: {:?}\n", self.stack) );
		try!( write!(f, "  Flags: C={:?} V={:?}\n", self.flag_c, self.flag_v) );
		try!( write!(f, "}}") );
		Ok( () )
	}
}

impl ::std::fmt::Display for StateData
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
	{
		for (i,reg) in self.registers.iter().enumerate()
		{
			if ! reg.is_unknown() {
				try!( write!(f, "  R{:2}={:?}", i, reg) );
			}
		}
		Ok( () )
	}
}

// vim: ft=rust
