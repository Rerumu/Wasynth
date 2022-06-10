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

macro_rules! leak_with_predicate {
	($name:tt, $predicate:tt) => {
		fn $name(&mut self, id: usize) {
			self.leak_with(|v| v.$predicate(id));
		}
	};
}

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

	fn arity_of(&self, index: usize) -> Arity {
		let Type::Function(typ) = &self.data[index];

		Arity::from_type(typ)
	}

	fn rel_arity_of(&self, index: usize) -> Arity {
		let adjusted = self
			.func_ex
			.iter()
			.chain(self.func_in.iter())
			.nth(index)
			.unwrap();

		self.arity_of(*adjusted)
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
struct StatList {
	code: Vec<Statement>,
	stack: Vec<Expression>,

	num_result: usize,
	num_stack: usize,
	is_else: bool,
}

impl StatList {
	fn new() -> Self {
		Self::default()
	}

	fn pop_required(&mut self) -> Expression {
		self.stack.pop().unwrap()
	}

	fn pop_len(&mut self, len: usize) -> Vec<Expression> {
		self.stack.split_off(self.stack.len() - len)
	}

	fn push_tracked(&mut self, data: Expression) {
		self.stack.push(data);
		self.num_stack = self.num_stack.max(self.stack.len());
	}

	fn leak_at(&mut self, var: usize) {
		let old = self.stack.get_mut(var).unwrap();

		if old.is_temporary(var) {
			return;
		}

		let get = Expression::GetTemporary(GetTemporary { var });
		let set = Statement::SetTemporary(SetTemporary {
			var,
			value: std::mem::replace(old, get),
		});

		self.code.push(set);
	}

	fn leak_with<P>(&mut self, predicate: P)
	where
		P: Fn(&Expression) -> bool,
	{
		let pend: Vec<_> = self
			.stack
			.iter()
			.enumerate()
			.filter_map(|v| predicate(v.1).then(|| v.0))
			.collect();

		for var in pend {
			self.leak_at(var);
		}
	}

	leak_with_predicate!(leak_local_write, is_local_read);
	leak_with_predicate!(leak_global_write, is_global_read);
	leak_with_predicate!(leak_memory_size, is_memory_size);
	leak_with_predicate!(leak_memory_write, is_memory_ref);

	fn leak_all(&mut self) {
		self.leak_with(|_| true);
	}

	fn push_temporary(&mut self, num: usize) {
		let len = self.stack.len();

		for var in len..len + num {
			let data = Expression::GetTemporary(GetTemporary { var });

			self.push_tracked(data);
		}
	}

	fn push_load(&mut self, what: LoadType, offset: u32) {
		let data = Expression::LoadAt(LoadAt {
			what,
			offset,
			pointer: self.pop_required().into(),
		});

		self.push_tracked(data);
	}

	fn add_store(&mut self, what: StoreType, offset: u32) {
		let data = Statement::StoreAt(StoreAt {
			what,
			offset,
			value: self.pop_required(),
			pointer: self.pop_required(),
		});

		self.leak_memory_write(0);
		self.code.push(data);
	}

	fn push_constant<T: Into<Value>>(&mut self, value: T) {
		let value = Expression::Value(value.into());

		self.push_tracked(value);
	}

	fn push_un_op(&mut self, op: UnOpType) {
		let data = Expression::UnOp(UnOp {
			op,
			rhs: self.pop_required().into(),
		});

		self.push_tracked(data);
	}

	fn push_bin_op(&mut self, op: BinOpType) {
		let data = Expression::BinOp(BinOp {
			op,
			rhs: self.pop_required().into(),
			lhs: self.pop_required().into(),
		});

		self.push_tracked(data);
	}

	fn push_cmp_op(&mut self, op: CmpOpType) {
		let data = Expression::CmpOp(CmpOp {
			op,
			rhs: self.pop_required().into(),
			lhs: self.pop_required().into(),
		});

		self.push_tracked(data);
	}

	// Eqz is the only unary comparison so it's "emulated"
	// using a constant operand
	fn try_add_equal_zero(&mut self, inst: &Instruction) -> bool {
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

	// Try to generate a simple operation
	fn try_add_operation(&mut self, inst: &Instruction) -> bool {
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
			self.try_add_equal_zero(inst)
		}
	}
}

pub struct Builder<'a> {
	type_info: &'a TypeInfo<'a>,

	pending: Vec<StatList>,
	target: StatList,

	num_result: usize,
	nested_unreachable: usize,
}

impl<'a> Builder<'a> {
	#[must_use]
	pub fn from_type_info(type_info: &'a TypeInfo<'a>) -> Self {
		Self {
			type_info,
			pending: Vec::new(),
			target: StatList::new(),
			num_result: 0,
			nested_unreachable: 0,
		}
	}

	#[must_use]
	pub fn build_anonymous(mut self, list: &[Instruction]) -> Intermediate {
		let data = self.build_stat_list(list, 1);

		Intermediate {
			local_data: Vec::new(),
			num_param: 0,
			num_stack: data.num_stack,
			code: Forward { body: data.code },
		}
	}

	#[must_use]
	pub fn build_indexed(mut self, index: usize, func: &FuncBody) -> Intermediate {
		let arity = &self.type_info.rel_arity_of(self.type_info.len_ex() + index);
		let data = self.build_stat_list(func.code().elements(), arity.num_result);

		Intermediate {
			local_data: func.locals().to_vec(),
			num_param: arity.num_param,
			num_stack: data.num_stack,
			code: Forward { body: data.code },
		}
	}

	fn start_block(&mut self, typ: BlockType) {
		let mut old = std::mem::take(&mut self.target);

		self.target.push_temporary(old.stack.len());
		self.target.num_result = match typ {
			BlockType::NoResult => 0,
			BlockType::Value(_) => 1,
			BlockType::TypeIndex(i) => {
				let id = i.try_into().unwrap();

				self.type_info.arity_of(id).num_result
			}
		};

		old.leak_all();
		old.push_temporary(self.target.num_result);

		self.pending.push(old);
	}

	fn end_block(&mut self) {
		let old = self.pending.pop().unwrap();
		let now = std::mem::replace(&mut self.target, old);

		self.target.num_stack = now.num_stack;

		match self.target.code.last_mut().unwrap() {
			Statement::Forward(data) => data.body = now.code,
			Statement::Backward(data) => data.body = now.code,
			Statement::If(data) if now.is_else => data.falsey = now.code,
			Statement::If(data) if !now.is_else => data.truthy = now.code,
			_ => unreachable!(),
		}
	}

	fn add_call(&mut self, func: usize) {
		let arity = self.type_info.rel_arity_of(func);
		let param_list = self.target.pop_len(arity.num_param);

		let first = self.target.stack.len();
		let result = first..first + arity.num_result;

		self.target.leak_all();
		self.target.push_temporary(arity.num_result);

		let data = Statement::Call(Call {
			func,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	fn add_call_indirect(&mut self, typ: usize, table: usize) {
		let arity = self.type_info.arity_of(typ);
		let index = self.target.pop_required();
		let param_list = self.target.pop_len(arity.num_param);

		let first = self.target.stack.len();
		let result = first..first + arity.num_result;

		self.target.leak_all();
		self.target.push_temporary(arity.num_result);

		let data = Statement::CallIndirect(CallIndirect {
			table,
			index,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	fn add_return(&mut self) {
		let data = Statement::Return(Return {
			list: self.target.pop_len(self.num_result),
		});

		self.target.leak_all();
		self.target.code.push(data);
	}

	#[cold]
	fn drop_unreachable(&mut self, inst: &Instruction) {
		match inst {
			Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => {
				self.nested_unreachable += 1;
			}
			Instruction::End => {
				self.nested_unreachable -= 1;

				if self.nested_unreachable == 0 {
					self.end_block();
				}
			}
			_ => {}
		}
	}

	fn add_instruction(&mut self, inst: &Instruction) {
		use Instruction as Inst;

		if self.target.try_add_operation(inst) {
			return;
		}

		match *inst {
			Inst::Unreachable => {
				self.nested_unreachable += 1;
				self.target.code.push(Statement::Unreachable);
			}
			Inst::Nop => {}
			Inst::Block(typ) => {
				let data = Statement::Forward(Forward { body: Vec::new() });

				self.start_block(typ);
				self.pending.last_mut().unwrap().code.push(data);
			}
			Inst::Loop(typ) => {
				let data = Statement::Backward(Backward { body: Vec::new() });

				self.start_block(typ);
				self.pending.last_mut().unwrap().code.push(data);
			}
			Inst::If(typ) => {
				let data = Statement::If(If {
					cond: self.target.pop_required(),
					truthy: Vec::new(),
					falsey: Vec::new(),
				});

				self.start_block(typ);
				self.pending.last_mut().unwrap().code.push(data);
			}
			Inst::Else => {
				let num_result = self.target.num_result;

				self.target.leak_all();
				self.end_block();
				self.start_block(BlockType::NoResult);

				self.target.num_result = num_result;
				self.target.is_else = true;
			}
			Inst::End => {
				self.target.leak_all();
				self.end_block();
			}
			Inst::Br(v) => {
				self.nested_unreachable += 1;

				let data = Statement::Br(Br {
					target: v.try_into().unwrap(),
				});

				self.target.leak_all();
				self.target.code.push(data);
			}
			Inst::BrIf(v) => {
				let data = Statement::BrIf(BrIf {
					cond: self.target.pop_required(),
					target: v.try_into().unwrap(),
				});

				// FIXME: Does not push results unless true
				// self.target.add_result_data();
				self.target.code.push(data);
			}
			Inst::BrTable(ref v) => {
				self.nested_unreachable += 1;

				let data = Statement::BrTable(BrTable {
					cond: self.target.pop_required(),
					data: *v.clone(),
				});

				self.target.leak_all();
				self.target.code.push(data);
			}
			Inst::Return => {
				self.nested_unreachable += 1;
				self.add_return();
			}
			Inst::Call(i) => {
				self.add_call(i.try_into().unwrap());
			}
			Inst::CallIndirect(i, t) => {
				self.add_call_indirect(i.try_into().unwrap(), t.into());
			}
			Inst::Drop => {
				let last = self.target.stack.len() - 1;

				if self.target.stack[last].has_side_effect() {
					self.target.leak_at(last);
				}

				self.target.pop_required();
			}
			Inst::Select => {
				let data = Expression::Select(Select {
					cond: self.target.pop_required().into(),
					b: self.target.pop_required().into(),
					a: self.target.pop_required().into(),
				});

				self.target.push_tracked(data);
			}
			Inst::GetLocal(i) => {
				let data = Expression::GetLocal(GetLocal {
					var: i.try_into().unwrap(),
				});

				self.target.push_tracked(data);
			}
			Inst::SetLocal(i) => {
				let var = i.try_into().unwrap();
				let data = Statement::SetLocal(SetLocal {
					var,
					value: self.target.pop_required(),
				});

				self.target.leak_local_write(var);
				self.target.code.push(data);
			}
			Inst::TeeLocal(i) => {
				let var = i.try_into().unwrap();
				let get = Expression::GetLocal(GetLocal { var });
				let set = Statement::SetLocal(SetLocal {
					var,
					value: self.target.pop_required(),
				});

				self.target.leak_local_write(var);
				self.target.push_tracked(get);
				self.target.code.push(set);
			}
			Inst::GetGlobal(i) => {
				let data = Expression::GetGlobal(GetGlobal {
					var: i.try_into().unwrap(),
				});

				self.target.push_tracked(data);
			}
			Inst::SetGlobal(i) => {
				let var = i.try_into().unwrap();
				let data = Statement::SetGlobal(SetGlobal {
					var,
					value: self.target.pop_required(),
				});

				self.target.leak_global_write(var);
				self.target.code.push(data);
			}
			Inst::I32Load(_, o) => self.target.push_load(LoadType::I32, o),
			Inst::I64Load(_, o) => self.target.push_load(LoadType::I64, o),
			Inst::F32Load(_, o) => self.target.push_load(LoadType::F32, o),
			Inst::F64Load(_, o) => self.target.push_load(LoadType::F64, o),
			Inst::I32Load8S(_, o) => self.target.push_load(LoadType::I32_I8, o),
			Inst::I32Load8U(_, o) => self.target.push_load(LoadType::I32_U8, o),
			Inst::I32Load16S(_, o) => self.target.push_load(LoadType::I32_I16, o),
			Inst::I32Load16U(_, o) => self.target.push_load(LoadType::I32_U16, o),
			Inst::I64Load8S(_, o) => self.target.push_load(LoadType::I64_I8, o),
			Inst::I64Load8U(_, o) => self.target.push_load(LoadType::I64_U8, o),
			Inst::I64Load16S(_, o) => self.target.push_load(LoadType::I64_I16, o),
			Inst::I64Load16U(_, o) => self.target.push_load(LoadType::I64_U16, o),
			Inst::I64Load32S(_, o) => self.target.push_load(LoadType::I64_I32, o),
			Inst::I64Load32U(_, o) => self.target.push_load(LoadType::I64_U32, o),
			Inst::I32Store(_, o) => self.target.add_store(StoreType::I32, o),
			Inst::I64Store(_, o) => self.target.add_store(StoreType::I64, o),
			Inst::F32Store(_, o) => self.target.add_store(StoreType::F32, o),
			Inst::F64Store(_, o) => self.target.add_store(StoreType::F64, o),
			Inst::I32Store8(_, o) => self.target.add_store(StoreType::I32_N8, o),
			Inst::I32Store16(_, o) => self.target.add_store(StoreType::I32_N16, o),
			Inst::I64Store8(_, o) => self.target.add_store(StoreType::I64_N8, o),
			Inst::I64Store16(_, o) => self.target.add_store(StoreType::I64_N16, o),
			Inst::I64Store32(_, o) => self.target.add_store(StoreType::I64_N32, o),
			Inst::CurrentMemory(i) => {
				let memory = i.try_into().unwrap();
				let data = Expression::MemorySize(MemorySize { memory });

				self.target.leak_memory_write(memory);
				self.target.push_tracked(data);
			}
			Inst::GrowMemory(i) => {
				let memory = i.try_into().unwrap();
				let data = Expression::MemoryGrow(MemoryGrow {
					memory,
					value: self.target.pop_required().into(),
				});

				self.target.leak_memory_size(memory);
				self.target.leak_memory_write(memory);
				self.target.push_tracked(data);
			}
			Inst::I32Const(v) => self.target.push_constant(v),
			Inst::I64Const(v) => self.target.push_constant(v),
			Inst::F32Const(v) => self.target.push_constant(v),
			Inst::F64Const(v) => self.target.push_constant(v),
			Inst::SignExt(_) => todo!(),
			_ => unreachable!(),
		}
	}

	fn build_stat_list(&mut self, list: &[Instruction], num_result: usize) -> StatList {
		self.nested_unreachable = 0;
		self.num_result = num_result;

		for inst in list.iter().take(list.len() - 1) {
			if self.nested_unreachable == 0 {
				self.add_instruction(inst);
			} else {
				self.drop_unreachable(inst);
			}
		}

		if self.nested_unreachable == 0 && num_result != 0 {
			self.add_return();
		}

		std::mem::take(&mut self.target)
	}
}
