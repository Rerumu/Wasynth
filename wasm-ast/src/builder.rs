use parity_wasm::elements::{
	BlockType, External, Func, FuncBody, FunctionSection, FunctionType, ImportEntry, ImportSection,
	Instruction, Module, Type, TypeSection,
};

use crate::{
	node::{
		Backward, BinOp, BinOpType, Br, BrIf, BrTable, Call, CallIndirect, CmpOp, CmpOpType,
		Expression, Forward, FuncData, GetGlobal, GetLocal, If, LoadAt, LoadType, MemoryGrow,
		MemorySize, Select, SetGlobal, SetLocal, Statement, StoreAt, StoreType, Terminator, UnOp,
		UnOpType, Value,
	},
	stack::{ReadType, Stack},
};

macro_rules! leak_on {
	($name:tt, $variant:tt) => {
		fn $name(&mut self, id: usize) {
			let read = ReadType::$variant(id);

			for i in 0..self.stack.len() {
				if self.stack.has_read_at(i, read) {
					self.leak_at(i);
				}
			}
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

	fn block_arity_of(&self, typ: BlockType) -> Arity {
		match typ {
			BlockType::NoResult => Arity {
				num_param: 0,
				num_result: 0,
			},
			BlockType::Value(_) => Arity {
				num_param: 0,
				num_result: 1,
			},
			BlockType::TypeIndex(i) => {
				let id = i.try_into().unwrap();

				self.arity_of(id)
			}
		}
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

#[derive(Clone, Copy)]
enum BlockVariant {
	Forward,
	Backward,
	If,
	Else,
}

enum BlockData {
	Forward { num_result: usize },
	Backward { num_param: usize },
	If { num_result: usize, typ: BlockType },
	Else { num_result: usize },
}

impl Default for BlockData {
	fn default() -> Self {
		BlockData::Forward { num_result: 0 }
	}
}

#[derive(Default)]
struct StatList {
	stack: Stack,
	code: Vec<Statement>,
	last: Option<Terminator>,

	block_data: BlockData,
}

impl StatList {
	fn new() -> Self {
		Self::default()
	}

	fn leak_at(&mut self, index: usize) {
		if let Some(set) = self.stack.leak_at(index) {
			self.code.push(set);
		}
	}

	fn leak_all(&mut self) {
		for i in 0..self.stack.len() {
			self.leak_at(i);
		}
	}

	leak_on!(leak_local_write, Local);
	leak_on!(leak_global_write, Global);
	leak_on!(leak_memory_write, Memory);

	fn push_load(&mut self, what: LoadType, offset: u32) {
		let data = Expression::LoadAt(LoadAt {
			what,
			offset,
			pointer: self.stack.pop().into(),
		});

		self.stack.push_with_single(data);
	}

	fn add_store(&mut self, what: StoreType, offset: u32) {
		let data = Statement::StoreAt(StoreAt {
			what,
			offset,
			value: self.stack.pop(),
			pointer: self.stack.pop(),
		});

		self.leak_memory_write(0);
		self.code.push(data);
	}

	fn push_constant<T: Into<Value>>(&mut self, value: T) {
		let value = Expression::Value(value.into());

		self.stack.push(value);
	}

	fn push_un_op(&mut self, op: UnOpType) {
		let rhs = self.stack.pop_with_read();
		let data = Expression::UnOp(UnOp {
			op,
			rhs: rhs.0.into(),
		});

		self.stack.push_with_read(data, rhs.1);
	}

	fn push_bin_op(&mut self, op: BinOpType) {
		let mut rhs = self.stack.pop_with_read();
		let lhs = self.stack.pop_with_read();

		let data = Expression::BinOp(BinOp {
			op,
			rhs: rhs.0.into(),
			lhs: lhs.0.into(),
		});

		rhs.1.extend(lhs.1);

		self.stack.push_with_read(data, rhs.1);
	}

	fn push_cmp_op(&mut self, op: CmpOpType) {
		let mut rhs = self.stack.pop_with_read();
		let lhs = self.stack.pop_with_read();

		let data = Expression::CmpOp(CmpOp {
			op,
			rhs: rhs.0.into(),
			lhs: lhs.0.into(),
		});

		rhs.1.extend(lhs.1);

		self.stack.push_with_read(data, rhs.1);
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

	fn set_terminator(&mut self, term: Terminator) {
		self.leak_all();
		self.last = Some(term);
	}
}

impl From<StatList> for Forward {
	fn from(stat: StatList) -> Self {
		Self {
			code: stat.code,
			last: stat.last,
		}
	}
}

impl From<StatList> for Backward {
	fn from(stat: StatList) -> Self {
		Self {
			code: stat.code,
			last: stat.last,
		}
	}
}

pub struct Builder<'a> {
	type_info: &'a TypeInfo<'a>,

	pending: Vec<StatList>,
	target: StatList,

	nested_unreachable: usize,
}

impl<'a> Builder<'a> {
	#[must_use]
	pub fn from_type_info(type_info: &'a TypeInfo<'a>) -> Self {
		Self {
			type_info,
			pending: Vec::new(),
			target: StatList::new(),
			nested_unreachable: 0,
		}
	}

	#[must_use]
	pub fn build_anonymous(mut self, list: &[Instruction]) -> FuncData {
		let data = self.build_stat_list(list, 1);

		FuncData {
			local_data: Vec::new(),
			num_result: 1,
			num_param: 0,
			num_stack: data.stack.capacity,
			code: data.into(),
		}
	}

	#[must_use]
	pub fn build_indexed(mut self, index: usize, func: &FuncBody) -> FuncData {
		let arity = &self.type_info.rel_arity_of(self.type_info.len_ex() + index);
		let data = self.build_stat_list(func.code().elements(), arity.num_result);

		FuncData {
			local_data: func.locals().to_vec(),
			num_result: arity.num_result,
			num_param: arity.num_param,
			num_stack: data.stack.capacity,
			code: data.into(),
		}
	}

	fn start_block(&mut self, typ: BlockType, variant: BlockVariant) {
		let Arity {
			num_param,
			num_result,
		} = self.type_info.block_arity_of(typ);

		let mut old = std::mem::take(&mut self.target);

		old.leak_all();

		self.target.block_data = match variant {
			BlockVariant::Forward => BlockData::Forward { num_result },
			BlockVariant::Backward => BlockData::Backward { num_param },
			BlockVariant::If => BlockData::If { num_result, typ },
			BlockVariant::Else => {
				old.stack.pop_len(num_result).for_each(drop);
				old.stack.push_temporary(num_param);

				BlockData::Else { num_result }
			}
		};

		self.target.stack = old.stack.split_last(num_param);

		old.stack.push_temporary(num_result);

		self.pending.push(old);
	}

	fn start_else(&mut self) {
		let typ = if let BlockData::If { typ, .. } = self.target.block_data {
			typ
		} else {
			unreachable!()
		};

		self.target.leak_all();
		self.end_block();
		self.start_block(typ, BlockVariant::Else);
	}

	fn end_block(&mut self) {
		let old = self.pending.pop().unwrap();
		let now = std::mem::replace(&mut self.target, old);

		self.target.stack.capacity = now.stack.capacity;

		let stat = match now.block_data {
			BlockData::Forward { .. } => Statement::Forward(now.into()),
			BlockData::Backward { .. } => Statement::Backward(now.into()),
			BlockData::If { .. } => Statement::If(If {
				cond: self.target.stack.pop(),
				truthy: now.into(),
				falsey: None,
			}),
			BlockData::Else { .. } => {
				if let Statement::If(v) = self.target.code.last_mut().unwrap() {
					v.falsey = Some(now.into());
				} else {
					unreachable!()
				}

				return;
			}
		};

		self.target.code.push(stat);
	}

	fn get_relative_block(&self, index: usize) -> &StatList {
		if index == 0 {
			&self.target
		} else {
			&self.pending[self.pending.len() - index]
		}
	}

	fn get_br_terminator(&self, target: usize) -> Br {
		let block = self.get_relative_block(target);
		let par_result = match block.block_data {
			BlockData::Forward { num_result }
			| BlockData::If { num_result, .. }
			| BlockData::Else { num_result } => num_result,
			BlockData::Backward { num_param } => num_param,
		};

		let align = self
			.target
			.stack
			.get_br_alignment(block.stack.previous, par_result);

		Br { target, align }
	}

	fn add_call(&mut self, func: usize) {
		let arity = self.type_info.rel_arity_of(func);
		let param_list = self.target.stack.pop_len(arity.num_param).collect();

		let first = self.target.stack.len();
		let result = first..first + arity.num_result;

		self.target.leak_all();
		self.target.stack.push_temporary(arity.num_result);

		let data = Statement::Call(Call {
			func,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	fn add_call_indirect(&mut self, typ: usize, table: usize) {
		let arity = self.type_info.arity_of(typ);
		let index = self.target.stack.pop();
		let param_list = self.target.stack.pop_len(arity.num_param).collect();

		let first = self.target.stack.len();
		let result = first..first + arity.num_result;

		self.target.leak_all();
		self.target.stack.push_temporary(arity.num_result);

		let data = Statement::CallIndirect(CallIndirect {
			table,
			index,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	#[cold]
	fn drop_unreachable(&mut self, inst: &Instruction) {
		match inst {
			Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => {
				self.nested_unreachable += 1;
			}
			Instruction::Else if self.nested_unreachable == 1 => {
				self.nested_unreachable -= 1;

				self.start_else();
			}
			Instruction::End if self.nested_unreachable == 1 => {
				self.nested_unreachable -= 1;

				self.end_block();
			}
			Instruction::End => {
				self.nested_unreachable -= 1;
			}
			_ => {}
		}
	}

	#[allow(clippy::too_many_lines)]
	fn add_instruction(&mut self, inst: &Instruction) {
		use Instruction as Inst;

		if self.target.try_add_operation(inst) {
			return;
		}

		match *inst {
			Inst::Unreachable => {
				self.nested_unreachable += 1;

				self.target.set_terminator(Terminator::Unreachable);
			}
			Inst::Nop => {}
			Inst::Block(typ) => {
				self.start_block(typ, BlockVariant::Forward);
			}
			Inst::Loop(typ) => {
				self.start_block(typ, BlockVariant::Backward);
			}
			Inst::If(typ) => {
				let cond = self.target.stack.pop();

				self.start_block(typ, BlockVariant::If);
				self.pending.last_mut().unwrap().stack.push(cond);
			}
			Inst::Else => {
				self.start_else();
			}
			Inst::End => {
				self.target.leak_all();
				self.end_block();
			}
			Inst::Br(v) => {
				let target = v.try_into().unwrap();
				let term = Terminator::Br(self.get_br_terminator(target));

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Inst::BrIf(v) => {
				let data = Statement::BrIf(BrIf {
					cond: self.target.stack.pop(),
					target: self.get_br_terminator(v.try_into().unwrap()),
				});

				self.target.leak_all();
				self.target.code.push(data);
			}
			Inst::BrTable(ref v) => {
				let cond = self.target.stack.pop();
				let data = v
					.table
					.iter()
					.copied()
					.map(|v| self.get_br_terminator(v.try_into().unwrap()))
					.collect();

				let default = self.get_br_terminator(v.default.try_into().unwrap());

				let term = Terminator::BrTable(BrTable {
					cond,
					data,
					default,
				});

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Inst::Return => {
				let target = self.pending.len();
				let term = Terminator::Br(self.get_br_terminator(target));

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Inst::Call(i) => {
				self.add_call(i.try_into().unwrap());
			}
			Inst::CallIndirect(i, t) => {
				self.add_call_indirect(i.try_into().unwrap(), t.into());
			}
			Inst::Drop => {
				self.target.stack.pop();
			}
			Inst::Select => {
				let mut cond = self.target.stack.pop_with_read();
				let b = self.target.stack.pop_with_read();
				let a = self.target.stack.pop_with_read();

				let data = Expression::Select(Select {
					cond: cond.0.into(),
					b: b.0.into(),
					a: a.0.into(),
				});

				cond.1.extend(b.1);
				cond.1.extend(a.1);

				self.target.stack.push_with_read(data, cond.1);
			}
			Inst::GetLocal(i) => {
				let var = i.try_into().unwrap();
				let data = Expression::GetLocal(GetLocal { var });

				self.target.stack.push_with_single(data);
			}
			Inst::SetLocal(i) => {
				let var = i.try_into().unwrap();
				let data = Statement::SetLocal(SetLocal {
					var,
					value: self.target.stack.pop(),
				});

				self.target.leak_local_write(var);
				self.target.code.push(data);
			}
			Inst::TeeLocal(i) => {
				let var = i.try_into().unwrap();
				let get = Expression::GetLocal(GetLocal { var });
				let set = Statement::SetLocal(SetLocal {
					var,
					value: self.target.stack.pop(),
				});

				self.target.leak_local_write(var);
				self.target.stack.push_with_single(get);
				self.target.code.push(set);
			}
			Inst::GetGlobal(i) => {
				let var = i.try_into().unwrap();
				let data = Expression::GetGlobal(GetGlobal { var });

				self.target.stack.push_with_single(data);
			}
			Inst::SetGlobal(i) => {
				let var = i.try_into().unwrap();
				let data = Statement::SetGlobal(SetGlobal {
					var,
					value: self.target.stack.pop(),
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

				self.target.stack.push(data);
			}
			Inst::GrowMemory(i) => {
				let value = self.target.stack.pop().into();
				let result = self.target.stack.len();
				let memory = i.try_into().unwrap();

				let data = Statement::MemoryGrow(MemoryGrow {
					result,
					memory,
					value,
				});

				self.target.leak_memory_write(memory);
				self.target.stack.push_temporary(1);
				self.target.code.push(data);
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
		self.target.block_data = BlockData::Forward { num_result };
		self.nested_unreachable = 0;

		for inst in list.iter().take(list.len() - 1) {
			if self.nested_unreachable == 0 {
				self.add_instruction(inst);
			} else {
				self.drop_unreachable(inst);
			}
		}

		if self.nested_unreachable == 0 {
			self.target.leak_all();
		}

		std::mem::take(&mut self.target)
	}
}
