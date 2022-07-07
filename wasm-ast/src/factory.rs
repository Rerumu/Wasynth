use wasmparser::{BlockType, FunctionBody, MemoryImmediate, Operator};

use crate::{
	module::TypeInfo,
	node::{
		BinOp, BinOpType, Block, Br, BrIf, BrTable, Call, CallIndirect, CmpOp, CmpOpType,
		Expression, FuncData, GetGlobal, GetLocal, If, LabelType, LoadAt, LoadType, MemoryGrow,
		MemorySize, Select, SetGlobal, SetLocal, Statement, StoreAt, StoreType, Terminator, UnOp,
		UnOpType, Value,
	},
	stack::{ReadType, Stack},
};

macro_rules! leak_on {
	($name:tt, $variant:tt) => {
		fn $name(&mut self, id: usize) {
			let read = ReadType::$variant(id);

			self.stack.leak_into(&mut self.code, |v| v.has_read(read))
		}
	};
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
	If { num_result: usize, ty: BlockType },
	Else { num_result: usize },
}

impl Default for BlockData {
	fn default() -> Self {
		BlockData::Forward { num_result: 0 }
	}
}

impl From<BlockData> for LabelType {
	fn from(data: BlockData) -> Self {
		match data {
			BlockData::Forward { .. } | BlockData::If { .. } | BlockData::Else { .. } => {
				Self::Forward
			}
			BlockData::Backward { .. } => Self::Backward,
		}
	}
}

#[derive(Default)]
struct StatList {
	stack: Stack,
	code: Vec<Statement>,
	last: Option<Terminator>,

	block_data: BlockData,
	has_reference: bool,
}

impl StatList {
	fn new() -> Self {
		Self::default()
	}

	fn leak_all(&mut self) {
		self.stack.leak_into(&mut self.code, |_| true);
	}

	fn leak_pre_call(&mut self) {
		self.stack.leak_into(&mut self.code, |v| {
			v.has_global_read() || v.has_memory_read()
		});
	}

	leak_on!(leak_local_write, Local);
	leak_on!(leak_global_write, Global);
	leak_on!(leak_memory_write, Memory);

	fn push_load(&mut self, load_type: LoadType, memarg: MemoryImmediate) {
		let memory = memarg.memory.try_into().unwrap();
		let offset = memarg.offset.try_into().unwrap();

		let data = Expression::LoadAt(LoadAt {
			load_type,
			memory,
			offset,
			pointer: self.stack.pop().into(),
		});

		self.stack.push_with_single(data);
	}

	fn add_store(&mut self, store_type: StoreType, memarg: MemoryImmediate) {
		let memory = memarg.memory.try_into().unwrap();
		let offset = memarg.offset.try_into().unwrap();

		let data = Statement::StoreAt(StoreAt {
			store_type,
			memory,
			offset,
			value: self.stack.pop(),
			pointer: self.stack.pop(),
		});

		self.leak_memory_write(memory);
		self.code.push(data);
	}

	fn push_constant<T: Into<Value>>(&mut self, value: T) {
		let value = Expression::Value(value.into());

		self.stack.push(value);
	}

	fn push_un_op(&mut self, op_type: UnOpType) {
		let rhs = self.stack.pop_with_read();
		let data = Expression::UnOp(UnOp {
			op_type,
			rhs: rhs.0.into(),
		});

		self.stack.push_with_read(data, rhs.1);
	}

	fn push_bin_op(&mut self, op_type: BinOpType) {
		let mut rhs = self.stack.pop_with_read();
		let lhs = self.stack.pop_with_read();

		let data = Expression::BinOp(BinOp {
			op_type,
			rhs: rhs.0.into(),
			lhs: lhs.0.into(),
		});

		rhs.1.extend(lhs.1);

		self.stack.push_with_read(data, rhs.1);
	}

	fn push_cmp_op(&mut self, op_type: CmpOpType) {
		let mut rhs = self.stack.pop_with_read();
		let lhs = self.stack.pop_with_read();

		let data = Expression::CmpOp(CmpOp {
			op_type,
			rhs: rhs.0.into(),
			lhs: lhs.0.into(),
		});

		rhs.1.extend(lhs.1);

		self.stack.push_with_read(data, rhs.1);
	}

	// Eqz is the only unary comparison so it's "emulated"
	// using a constant operand
	fn try_add_equal_zero(&mut self, op: &Operator) -> bool {
		match op {
			Operator::I32Eqz => {
				self.push_constant(0_i32);
				self.push_cmp_op(CmpOpType::Eq_I32);

				true
			}
			Operator::I64Eqz => {
				self.push_constant(0_i64);
				self.push_cmp_op(CmpOpType::Eq_I64);

				true
			}
			_ => false,
		}
	}

	// Try to generate a simple operation
	fn try_add_operation(&mut self, op: &Operator) -> bool {
		if let Ok(op_type) = UnOpType::try_from(op) {
			self.push_un_op(op_type);

			true
		} else if let Ok(op_type) = BinOpType::try_from(op) {
			self.push_bin_op(op_type);

			true
		} else if let Ok(op_type) = CmpOpType::try_from(op) {
			self.push_cmp_op(op_type);

			true
		} else {
			self.try_add_equal_zero(op)
		}
	}

	fn set_terminator(&mut self, term: Terminator) {
		self.leak_all();
		self.last = Some(term);
	}
}

impl From<StatList> for Block {
	fn from(stat: StatList) -> Self {
		let label_type = stat.has_reference.then(|| stat.block_data.into());

		Self {
			label_type,
			code: stat.code,
			last: stat.last,
		}
	}
}

pub struct Factory<'a> {
	type_info: &'a TypeInfo<'a>,

	pending: Vec<StatList>,
	target: StatList,

	nested_unreachable: usize,
}

impl<'a> Factory<'a> {
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
	pub fn create_anonymous(&mut self, list: &[Operator]) -> FuncData {
		let data = self.build_stat_list(list, 1);

		FuncData {
			local_data: Vec::new(),
			num_result: 1,
			num_param: 0,
			num_stack: data.stack.capacity,
			code: data.into(),
		}
	}

	/// # Panics
	///
	/// Panics if the function is malformed.
	#[must_use]
	pub fn create_indexed(&mut self, index: usize, func: &FunctionBody) -> FuncData {
		let code: Result<Vec<_>, _> = func.get_operators_reader().unwrap().into_iter().collect();
		let local: Result<Vec<_>, _> = func.get_locals_reader().unwrap().into_iter().collect();

		let (num_param, num_result) = self.type_info.by_func_index(index);
		let data = self.build_stat_list(&code.unwrap(), num_result);

		FuncData {
			local_data: local.unwrap(),
			num_result,
			num_param,
			num_stack: data.stack.capacity,
			code: data.into(),
		}
	}

	fn start_block(&mut self, ty: BlockType, variant: BlockVariant) {
		let (num_param, num_result) = self.type_info.by_block_type(ty);
		let mut old = std::mem::take(&mut self.target);

		old.leak_all();

		self.target.block_data = match variant {
			BlockVariant::Forward => BlockData::Forward { num_result },
			BlockVariant::Backward => BlockData::Backward { num_param },
			BlockVariant::If => BlockData::If { num_result, ty },
			BlockVariant::Else => {
				old.stack.pop_len(num_result).for_each(drop);
				old.stack.push_temporary(num_param);

				BlockData::Else { num_result }
			}
		};

		self.target.stack = old.stack.split_last(num_param, num_result);

		old.stack.push_temporary(num_result);

		self.pending.push(old);
	}

	fn start_else(&mut self) {
		let ty = match self.target.block_data {
			BlockData::If { ty, .. } => ty,
			_ => unreachable!(),
		};

		self.target.leak_all();
		self.end_block();
		self.start_block(ty, BlockVariant::Else);
	}

	fn end_block(&mut self) {
		let old = self.pending.pop().unwrap();
		let now = std::mem::replace(&mut self.target, old);

		self.target.stack.capacity = now.stack.capacity;

		let stat = match now.block_data {
			BlockData::Forward { .. } | BlockData::Backward { .. } => Statement::Block(now.into()),
			BlockData::If { .. } => Statement::If(If {
				condition: self.target.stack.pop(),
				on_true: now.into(),
				on_false: None,
			}),
			BlockData::Else { .. } => {
				if let Statement::If(v) = self.target.code.last_mut().unwrap() {
					v.on_false = Some(now.into());
				} else {
					unreachable!()
				}

				return;
			}
		};

		self.target.code.push(stat);
	}

	fn get_relative_block(&mut self, index: usize) -> &mut StatList {
		if index == 0 {
			&mut self.target
		} else {
			let index = self.pending.len() - index;

			&mut self.pending[index]
		}
	}

	fn get_br_terminator(&mut self, target: usize) -> Br {
		let block = self.get_relative_block(target);
		let previous = block.stack.previous;
		let result = match block.block_data {
			BlockData::Forward { num_result }
			| BlockData::If { num_result, .. }
			| BlockData::Else { num_result } => num_result,
			BlockData::Backward { num_param } => num_param,
		};

		block.has_reference = true;

		let align = self.target.stack.get_br_alignment(previous, result);

		Br { target, align }
	}

	fn add_call(&mut self, function: usize) {
		let (num_param, num_result) = self.type_info.by_func_index(function);
		let param_list = self.target.stack.pop_len(num_param).collect();

		self.target.leak_pre_call();

		let result = self.target.stack.push_temporary(num_result);

		let data = Statement::Call(Call {
			function,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	fn add_call_indirect(&mut self, ty: usize, table: usize) {
		let (num_param, num_result) = self.type_info.by_type_index(ty);
		let index = self.target.stack.pop();
		let param_list = self.target.stack.pop_len(num_param).collect();

		self.target.leak_pre_call();

		let result = self.target.stack.push_temporary(num_result);

		let data = Statement::CallIndirect(CallIndirect {
			table,
			index,
			result,
			param_list,
		});

		self.target.code.push(data);
	}

	#[cold]
	fn drop_unreachable(&mut self, op: &Operator) {
		match op {
			Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
				self.nested_unreachable += 1;
			}
			Operator::Else if self.nested_unreachable == 1 => {
				self.nested_unreachable -= 1;

				self.start_else();
			}
			Operator::End if self.nested_unreachable == 1 => {
				self.nested_unreachable -= 1;

				self.end_block();
			}
			Operator::End => {
				self.nested_unreachable -= 1;
			}
			_ => {}
		}
	}

	#[allow(clippy::too_many_lines)]
	fn add_instruction(&mut self, op: &Operator) {
		if self.target.try_add_operation(op) {
			return;
		}

		match *op {
			Operator::Unreachable => {
				self.nested_unreachable += 1;

				self.target.set_terminator(Terminator::Unreachable);
			}
			Operator::Nop => {}
			Operator::Block { ty } => {
				self.start_block(ty, BlockVariant::Forward);
			}
			Operator::Loop { ty } => {
				self.start_block(ty, BlockVariant::Backward);
			}
			Operator::If { ty } => {
				let cond = self.target.stack.pop();

				self.start_block(ty, BlockVariant::If);
				self.pending.last_mut().unwrap().stack.push(cond);
			}
			Operator::Else => {
				self.start_else();
			}
			Operator::End => {
				self.target.leak_all();
				self.end_block();
			}
			Operator::Br { relative_depth } => {
				let target = relative_depth.try_into().unwrap();
				let term = Terminator::Br(self.get_br_terminator(target));

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Operator::BrIf { relative_depth } => {
				let target = relative_depth.try_into().unwrap();
				let data = Statement::BrIf(BrIf {
					condition: self.target.stack.pop(),
					target: self.get_br_terminator(target),
				});

				self.target.leak_all();
				self.target.code.push(data);
			}
			Operator::BrTable { ref table } => {
				let condition = self.target.stack.pop();
				let data = table
					.targets()
					.map(Result::unwrap)
					.map(|v| self.get_br_terminator(v.try_into().unwrap()))
					.collect();

				let default = self.get_br_terminator(table.default().try_into().unwrap());

				let term = Terminator::BrTable(BrTable {
					condition,
					data,
					default,
				});

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Operator::Return => {
				let target = self.pending.len();
				let term = Terminator::Br(self.get_br_terminator(target));

				self.target.set_terminator(term);
				self.nested_unreachable += 1;
			}
			Operator::Call { function_index } => {
				let index = function_index.try_into().unwrap();

				self.add_call(index);
			}
			Operator::CallIndirect {
				index, table_byte, ..
			} => {
				let index = index.try_into().unwrap();

				self.add_call_indirect(index, table_byte.into());
			}
			Operator::Drop => {
				self.target.stack.pop();
			}
			Operator::Select => {
				let mut condition = self.target.stack.pop_with_read();
				let on_false = self.target.stack.pop_with_read();
				let on_true = self.target.stack.pop_with_read();

				let data = Expression::Select(Select {
					condition: condition.0.into(),
					on_true: on_true.0.into(),
					on_false: on_false.0.into(),
				});

				condition.1.extend(on_true.1);
				condition.1.extend(on_false.1);

				self.target.stack.push_with_read(data, condition.1);
			}
			Operator::LocalGet { local_index } => {
				let var = local_index.try_into().unwrap();
				let data = Expression::GetLocal(GetLocal { var });

				self.target.stack.push_with_single(data);
			}
			Operator::LocalSet { local_index } => {
				let var = local_index.try_into().unwrap();
				let data = Statement::SetLocal(SetLocal {
					var,
					value: self.target.stack.pop(),
				});

				self.target.leak_local_write(var);
				self.target.code.push(data);
			}
			Operator::LocalTee { local_index } => {
				let var = local_index.try_into().unwrap();
				let get = Expression::GetLocal(GetLocal { var });
				let set = Statement::SetLocal(SetLocal {
					var,
					value: self.target.stack.pop(),
				});

				self.target.leak_local_write(var);
				self.target.stack.push_with_single(get);
				self.target.code.push(set);
			}
			Operator::GlobalGet { global_index } => {
				let var = global_index.try_into().unwrap();
				let data = Expression::GetGlobal(GetGlobal { var });

				self.target.stack.push_with_single(data);
			}
			Operator::GlobalSet { global_index } => {
				let var = global_index.try_into().unwrap();
				let data = Statement::SetGlobal(SetGlobal {
					var,
					value: self.target.stack.pop(),
				});

				self.target.leak_global_write(var);
				self.target.code.push(data);
			}
			Operator::I32Load { memarg } => self.target.push_load(LoadType::I32, memarg),
			Operator::I64Load { memarg } => self.target.push_load(LoadType::I64, memarg),
			Operator::F32Load { memarg } => self.target.push_load(LoadType::F32, memarg),
			Operator::F64Load { memarg } => self.target.push_load(LoadType::F64, memarg),
			Operator::I32Load8S { memarg } => self.target.push_load(LoadType::I32_I8, memarg),
			Operator::I32Load8U { memarg } => self.target.push_load(LoadType::I32_U8, memarg),
			Operator::I32Load16S { memarg } => self.target.push_load(LoadType::I32_I16, memarg),
			Operator::I32Load16U { memarg } => self.target.push_load(LoadType::I32_U16, memarg),
			Operator::I64Load8S { memarg } => self.target.push_load(LoadType::I64_I8, memarg),
			Operator::I64Load8U { memarg } => self.target.push_load(LoadType::I64_U8, memarg),
			Operator::I64Load16S { memarg } => self.target.push_load(LoadType::I64_I16, memarg),
			Operator::I64Load16U { memarg } => self.target.push_load(LoadType::I64_U16, memarg),
			Operator::I64Load32S { memarg } => self.target.push_load(LoadType::I64_I32, memarg),
			Operator::I64Load32U { memarg } => self.target.push_load(LoadType::I64_U32, memarg),
			Operator::I32Store { memarg } => self.target.add_store(StoreType::I32, memarg),
			Operator::I64Store { memarg } => self.target.add_store(StoreType::I64, memarg),
			Operator::F32Store { memarg } => self.target.add_store(StoreType::F32, memarg),
			Operator::F64Store { memarg } => self.target.add_store(StoreType::F64, memarg),
			Operator::I32Store8 { memarg } => self.target.add_store(StoreType::I32_N8, memarg),
			Operator::I32Store16 { memarg } => self.target.add_store(StoreType::I32_N16, memarg),
			Operator::I64Store8 { memarg } => self.target.add_store(StoreType::I64_N8, memarg),
			Operator::I64Store16 { memarg } => self.target.add_store(StoreType::I64_N16, memarg),
			Operator::I64Store32 { memarg } => self.target.add_store(StoreType::I64_N32, memarg),
			Operator::MemorySize { mem, .. } => {
				let memory = mem.try_into().unwrap();
				let data = Expression::MemorySize(MemorySize { memory });

				self.target.stack.push(data);
			}
			Operator::MemoryGrow { mem, .. } => {
				let size = self.target.stack.pop().into();
				let result = self.target.stack.push_temporary(1).start;
				let memory = mem.try_into().unwrap();

				let data = Statement::MemoryGrow(MemoryGrow {
					memory,
					result,
					size,
				});

				self.target.leak_memory_write(memory);
				self.target.code.push(data);
			}
			Operator::I32Const { value } => self.target.push_constant(value),
			Operator::I64Const { value } => self.target.push_constant(value),
			Operator::F32Const { value } => self.target.push_constant(value.bits()),
			Operator::F64Const { value } => self.target.push_constant(value.bits()),
			_ => unimplemented!(),
		}
	}

	fn build_stat_list(&mut self, list: &[Operator], num_result: usize) -> StatList {
		self.target.block_data = BlockData::Forward { num_result };
		self.nested_unreachable = 0;

		for op in list.iter().take(list.len() - 1) {
			if self.nested_unreachable == 0 {
				self.add_instruction(op);
			} else {
				self.drop_unreachable(op);
			}
		}

		if self.nested_unreachable == 0 {
			self.target.leak_all();
		}

		std::mem::take(&mut self.target)
	}
}
