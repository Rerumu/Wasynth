use parity_wasm::elements::{
	BlockType, External, Func, FuncBody, FunctionSection, FunctionType, ImportEntry, ImportSection,
	Instruction, Module, Type, TypeSection,
};

use crate::node::{
	Backward, BinOp, BinOpType, Br, BrIf, BrTable, Call, CallIndirect, CmpOp, CmpOpType,
	Expression, Forward, GetGlobal, GetLocal, GetTemporary, If, Intermediate, LoadAt, LoadType,
	MemoryGrow, MemorySize, Return, Select, SetGlobal, SetLocal, SetTemporary, Statement, StoreAt,
	StoreType, UnOp, UnOpType, Value,
};

struct Arity {
	num_param: usize,
	num_result: usize,
}

impl Arity {
	fn from_type(typ: &FunctionType) -> Self {
		Self {
			num_param: typ.params().len(),
			num_result: typ.results().len(),
		}
	}
}

pub struct TypeInfo<'a> {
	data: &'a [Type],
	func_ex: Vec<usize>,
	func_in: Vec<usize>,
}

impl<'a> TypeInfo<'a> {
	#[must_use]
	pub fn from_module(parent: &'a Module) -> Self {
		let data = parent
			.type_section()
			.map_or([].as_ref(), TypeSection::types);

		let func_ex = Self::new_ex_list(parent);
		let func_in = Self::new_in_list(parent);

		Self {
			data,
			func_ex,
			func_in,
		}
	}

	#[must_use]
	pub fn len_in(&self) -> usize {
		self.func_in.len()
	}

	#[must_use]
	pub fn len_ex(&self) -> usize {
		self.func_ex.len()
	}

	fn raw_arity_of(&self, index: usize) -> Arity {
		let Type::Function(typ) = &self.data[index];

		Arity::from_type(typ)
	}

	fn arity_of(&self, index: usize) -> Arity {
		let adjusted = self
			.func_ex
			.iter()
			.chain(self.func_in.iter())
			.nth(index)
			.unwrap();

		self.raw_arity_of(*adjusted)
	}

	fn func_of_import(import: &ImportEntry) -> Option<usize> {
		if let &External::Function(i) = import.external() {
			Some(i.try_into().unwrap())
		} else {
			None
		}
	}

	fn new_ex_list(wasm: &Module) -> Vec<usize> {
		let list = wasm
			.import_section()
			.map_or([].as_ref(), ImportSection::entries);

		list.iter().filter_map(Self::func_of_import).collect()
	}

	fn new_in_list(wasm: &Module) -> Vec<usize> {
		let list = wasm
			.function_section()
			.map_or([].as_ref(), FunctionSection::entries);

		list.iter()
			.map(Func::type_ref)
			.map(|v| v.try_into().unwrap())
			.collect()
	}
}

#[derive(Default)]
struct Stacked {
	pending_list: Vec<Vec<Expression>>,
	stack: Vec<Expression>,
	last_stack: usize,
}

impl Stacked {
	fn new() -> Self {
		Self::default()
	}

	// If any expressions are still pending at the start of
	// statement, we leak them into variables.
	// Since expressions do not have set ordering rules, this is
	// safe and condenses code.
	fn gen_leak_pending(&mut self, stat: &mut Vec<Statement>) {
		self.last_stack = self.last_stack.max(self.stack.len());

		for (i, v) in self
			.stack
			.iter_mut()
			.enumerate()
			.filter(|v| !v.1.is_temporary(v.0))
		{
			let get = Expression::GetTemporary(GetTemporary { var: i });
			let set = Statement::SetTemporary(SetTemporary {
				var: i,
				value: std::mem::replace(v, get),
			});

			stat.push(set);
		}
	}

	// Pending expressions are put to sleep before entering
	// a control structure so that they are not lost.
	fn save_pending(&mut self) {
		let cloned = self.stack.iter().map(Expression::clone_temporary).collect();

		self.pending_list.push(cloned);
	}

	fn load_pending(&mut self) {
		self.stack = self.pending_list.pop().unwrap();
	}

	fn pop(&mut self) -> Expression {
		self.stack.pop().unwrap()
	}

	fn pop_many(&mut self, len: usize) -> Vec<Expression> {
		self.stack.split_off(self.stack.len() - len)
	}

	fn push(&mut self, value: Expression) {
		self.stack.push(value);
	}

	fn push_constant<T: Into<Value>>(&mut self, value: T) {
		let value = Expression::Value(value.into());

		self.stack.push(value);
	}

	fn push_temporary(&mut self, num: usize) {
		let len = self.stack.len();

		for var in len..len + num {
			self.stack
				.push(Expression::GetTemporary(GetTemporary { var }));
		}
	}

	fn push_load(&mut self, what: LoadType, offset: u32) {
		let pointer = Box::new(self.pop());

		self.stack.push(Expression::LoadAt(LoadAt {
			what,
			offset,
			pointer,
		}));
	}

	fn gen_store(&mut self, what: StoreType, offset: u32, stat: &mut Vec<Statement>) {
		let value = self.pop();
		let pointer = self.pop();

		self.gen_leak_pending(stat);

		stat.push(Statement::StoreAt(StoreAt {
			what,
			offset,
			pointer,
			value,
		}));
	}

	fn push_un_op(&mut self, op: UnOpType) {
		let rhs = Box::new(self.pop());

		self.stack.push(Expression::UnOp(UnOp { op, rhs }));
	}

	fn push_bin_op(&mut self, op: BinOpType) {
		let rhs = Box::new(self.pop());
		let lhs = Box::new(self.pop());

		self.stack.push(Expression::BinOp(BinOp { op, lhs, rhs }));
	}

	fn push_cmp_op(&mut self, op: CmpOpType) {
		let rhs = Box::new(self.pop());
		let lhs = Box::new(self.pop());

		self.stack.push(Expression::CmpOp(CmpOp { op, lhs, rhs }));
	}

	// Since Eqz is the only unary comparison it's cleaner to
	// generate a simple CmpOp
	fn try_equal_zero(&mut self, inst: &Instruction) -> bool {
		match inst {
			Instruction::I32Eqz => {
				self.push_constant(0_i32);
				self.push_cmp_op(CmpOpType::Eq_I32);

				true
			}
			Instruction::I64Eqz => {
				self.push_constant(0_i64);
				self.push_cmp_op(CmpOpType::Eq_I64);

				true
			}
			_ => false,
		}
	}

	fn try_operation(&mut self, inst: &Instruction) -> bool {
		if let Ok(op) = UnOpType::try_from(inst) {
			self.push_un_op(op);

			true
		} else if let Ok(op) = BinOpType::try_from(inst) {
			self.push_bin_op(op);

			true
		} else if let Ok(op) = CmpOpType::try_from(inst) {
			self.push_cmp_op(op);

			true
		} else {
			self.try_equal_zero(inst)
		}
	}
}

pub struct Builder<'a> {
	// target state
	type_info: &'a TypeInfo<'a>,
	num_result: usize,

	// translation state
	data: Stacked,
}

fn is_else_stat(inst: &Instruction) -> bool {
	inst == &Instruction::Else
}

fn is_dead_precursor(inst: &Instruction) -> bool {
	matches!(
		inst,
		Instruction::Unreachable | Instruction::Br(_) | Instruction::Return
	)
}

impl<'a> Builder<'a> {
	#[must_use]
	pub fn new(info: &'a TypeInfo) -> Builder<'a> {
		Builder {
			type_info: info,
			num_result: 0,
			data: Stacked::new(),
		}
	}

	#[must_use]
	pub fn with_anon(mut self, mut inst: &[Instruction]) -> Intermediate {
		self.num_result = 1;

		let code = self.new_forward(&mut inst);
		let num_stack = self.data.last_stack;

		Intermediate {
			local_data: Vec::new(),
			num_param: 0,
			num_stack,
			code,
		}
	}

	#[must_use]
	pub fn with_index(mut self, index: usize, func: &'a FuncBody) -> Intermediate {
		let arity = &self.type_info.arity_of(self.type_info.len_ex() + index);

		self.num_result = arity.num_result;

		let code = self.new_forward(&mut func.code().elements());
		let num_stack = self.data.last_stack;

		Intermediate {
			local_data: func.locals().to_vec(),
			num_param: arity.num_param,
			num_stack,
			code,
		}
	}

	fn push_block_result(&mut self, typ: BlockType) {
		let num = match typ {
			BlockType::NoResult => {
				return;
			}
			BlockType::Value(_) => 1,
			BlockType::TypeIndex(i) => {
				self.type_info
					.raw_arity_of(i.try_into().unwrap())
					.num_result
			}
		};

		self.data.push_temporary(num);
	}

	fn gen_return(&mut self, stat: &mut Vec<Statement>) {
		let list = self.data.pop_many(self.num_result);

		self.data.gen_leak_pending(stat);

		stat.push(Statement::Return(Return { list }));
	}

	fn gen_call(&mut self, func: usize, stat: &mut Vec<Statement>) {
		let arity = self.type_info.arity_of(func);
		let param_list = self.data.pop_many(arity.num_param);

		let first = self.data.stack.len();
		let result = first..first + arity.num_result;

		self.data.push_temporary(arity.num_result);
		self.data.gen_leak_pending(stat);

		stat.push(Statement::Call(Call {
			func,
			result,
			param_list,
		}));
	}

	fn gen_call_indirect(&mut self, typ: usize, table: usize, stat: &mut Vec<Statement>) {
		let arity = self.type_info.raw_arity_of(typ);
		let index = self.data.pop();
		let param_list = self.data.pop_many(arity.num_param);

		let first = self.data.stack.len();
		let result = first..first + arity.num_result;

		self.data.push_temporary(arity.num_result);
		self.data.gen_leak_pending(stat);

		stat.push(Statement::CallIndirect(CallIndirect {
			table,
			index,
			result,
			param_list,
		}));
	}

	fn drop_unreachable(list: &mut &[Instruction]) {
		use Instruction as Inst;

		let mut level = 1;

		loop {
			let inst = &list[0];

			*list = &list[1..];

			match inst {
				Inst::Block(_) | Inst::Loop(_) | Inst::If(_) => {
					level += 1;
				}
				Inst::Else => {
					if level == 1 {
						break;
					}
				}
				Inst::End => {
					level -= 1;

					if level == 0 {
						break;
					}
				}
				_ => {}
			}
		}
	}

	#[allow(clippy::too_many_lines)]
	fn new_stored_body(&mut self, list: &mut &[Instruction]) -> Vec<Statement> {
		use Instruction as Inst;

		let mut stat = Vec::new();

		self.data.save_pending();

		loop {
			let inst = &list[0];

			*list = &list[1..];

			if self.data.try_operation(inst) {
				continue;
			}

			match *inst {
				Inst::Nop => {}
				Inst::Unreachable => {
					stat.push(Statement::Unreachable);
				}
				Inst::Block(t) => {
					self.data.gen_leak_pending(&mut stat);

					let data = self.new_forward(list);

					self.push_block_result(t);
					stat.push(Statement::Forward(data));
				}
				Inst::Loop(t) => {
					self.data.gen_leak_pending(&mut stat);

					let data = self.new_backward(list);

					self.push_block_result(t);
					stat.push(Statement::Backward(data));
				}
				Inst::If(t) => {
					let cond = self.data.pop();

					self.data.gen_leak_pending(&mut stat);

					let data = self.new_if(cond, list);

					self.push_block_result(t);
					stat.push(Statement::If(data));
				}
				Inst::Else => {
					self.data.gen_leak_pending(&mut stat);

					break;
				}
				Inst::End => {
					if list.is_empty() && self.num_result != 0 {
						self.gen_return(&mut stat);
					} else {
						self.data.gen_leak_pending(&mut stat);
					}

					break;
				}
				Inst::Br(target) => {
					let target = target.try_into().unwrap();

					self.data.gen_leak_pending(&mut stat);
					stat.push(Statement::Br(Br { target }));
				}
				Inst::BrIf(target) => {
					let target = target.try_into().unwrap();
					let cond = self.data.pop();

					self.data.gen_leak_pending(&mut stat);
					stat.push(Statement::BrIf(BrIf { cond, target }));
				}
				Inst::BrTable(ref t) => {
					let cond = self.data.pop();

					self.data.gen_leak_pending(&mut stat);
					stat.push(Statement::BrTable(BrTable {
						cond,
						data: *t.clone(),
					}));
				}
				Inst::Return => {
					self.gen_return(&mut stat);
				}
				Inst::Call(i) => {
					self.gen_call(i.try_into().unwrap(), &mut stat);
				}
				Inst::CallIndirect(i, t) => {
					self.gen_call_indirect(i.try_into().unwrap(), t.into(), &mut stat);
				}
				Inst::Drop => {
					self.data.pop();
				}
				Inst::Select => {
					let cond = Box::new(self.data.pop());
					let b = Box::new(self.data.pop());
					let a = Box::new(self.data.pop());

					self.data.push(Expression::Select(Select { cond, a, b }));
				}
				Inst::GetLocal(var) => {
					let var = var.try_into().unwrap();

					self.data.push(Expression::GetLocal(GetLocal { var }));
				}
				Inst::SetLocal(var) => {
					let var = var.try_into().unwrap();
					let value = self.data.pop();

					self.data.gen_leak_pending(&mut stat);
					stat.push(Statement::SetLocal(SetLocal { var, value }));
				}
				Inst::TeeLocal(var) => {
					self.data.gen_leak_pending(&mut stat);

					let var = var.try_into().unwrap();
					let value = self.data.pop();

					self.data.push(value.clone_temporary());
					stat.push(Statement::SetLocal(SetLocal { var, value }));
				}
				Inst::GetGlobal(var) => {
					let var = var.try_into().unwrap();

					self.data.push(Expression::GetGlobal(GetGlobal { var }));
				}
				Inst::SetGlobal(var) => {
					let var = var.try_into().unwrap();
					let value = self.data.pop();

					stat.push(Statement::SetGlobal(SetGlobal { var, value }));
				}
				Inst::I32Load(_, o) => self.data.push_load(LoadType::I32, o),
				Inst::I64Load(_, o) => self.data.push_load(LoadType::I64, o),
				Inst::F32Load(_, o) => self.data.push_load(LoadType::F32, o),
				Inst::F64Load(_, o) => self.data.push_load(LoadType::F64, o),
				Inst::I32Load8S(_, o) => self.data.push_load(LoadType::I32_I8, o),
				Inst::I32Load8U(_, o) => self.data.push_load(LoadType::I32_U8, o),
				Inst::I32Load16S(_, o) => self.data.push_load(LoadType::I32_I16, o),
				Inst::I32Load16U(_, o) => self.data.push_load(LoadType::I32_U16, o),
				Inst::I64Load8S(_, o) => self.data.push_load(LoadType::I64_I8, o),
				Inst::I64Load8U(_, o) => self.data.push_load(LoadType::I64_U8, o),
				Inst::I64Load16S(_, o) => self.data.push_load(LoadType::I64_I16, o),
				Inst::I64Load16U(_, o) => self.data.push_load(LoadType::I64_U16, o),
				Inst::I64Load32S(_, o) => self.data.push_load(LoadType::I64_I32, o),
				Inst::I64Load32U(_, o) => self.data.push_load(LoadType::I64_U32, o),
				Inst::I32Store(_, o) => self.data.gen_store(StoreType::I32, o, &mut stat),
				Inst::I64Store(_, o) => self.data.gen_store(StoreType::I64, o, &mut stat),
				Inst::F32Store(_, o) => self.data.gen_store(StoreType::F32, o, &mut stat),
				Inst::F64Store(_, o) => self.data.gen_store(StoreType::F64, o, &mut stat),
				Inst::I32Store8(_, o) => self.data.gen_store(StoreType::I32_N8, o, &mut stat),
				Inst::I32Store16(_, o) => self.data.gen_store(StoreType::I32_N16, o, &mut stat),
				Inst::I64Store8(_, o) => self.data.gen_store(StoreType::I64_N8, o, &mut stat),
				Inst::I64Store16(_, o) => self.data.gen_store(StoreType::I64_N16, o, &mut stat),
				Inst::I64Store32(_, o) => self.data.gen_store(StoreType::I64_N32, o, &mut stat),
				Inst::CurrentMemory(memory) => {
					let memory = memory.try_into().unwrap();

					self.data
						.push(Expression::MemorySize(MemorySize { memory }));
				}
				Inst::GrowMemory(memory) => {
					let memory = memory.try_into().unwrap();
					let value = Box::new(self.data.pop());

					// `MemoryGrow` is an expression *but* it has side effects
					self.data
						.push(Expression::MemoryGrow(MemoryGrow { memory, value }));

					self.data.gen_leak_pending(&mut stat);
				}
				Inst::I32Const(v) => self.data.push_constant(v),
				Inst::I64Const(v) => self.data.push_constant(v),
				Inst::F32Const(v) => self.data.push_constant(v),
				Inst::F64Const(v) => self.data.push_constant(v),
				_ => unreachable!(),
			}

			if is_dead_precursor(inst) {
				Self::drop_unreachable(list);

				break;
			}
		}

		self.data.load_pending();

		stat
	}

	fn new_if(&mut self, cond: Expression, list: &mut &[Instruction]) -> If {
		let copied = <&[Instruction]>::clone(list);
		let truthy = self.new_stored_body(list);

		let end = copied.len() - list.len() - 1;
		let falsey = is_else_stat(&copied[end])
			.then(|| self.new_stored_body(list))
			.unwrap_or_default();

		If {
			cond,
			truthy,
			falsey,
		}
	}

	fn new_backward(&mut self, list: &mut &[Instruction]) -> Backward {
		Backward {
			body: self.new_stored_body(list),
		}
	}

	fn new_forward(&mut self, list: &mut &[Instruction]) -> Forward {
		Forward {
			body: self.new_stored_body(list),
		}
	}
}
