use parity_wasm::elements::{BlockType, Instruction, Local, Module};

use crate::backend::translator::arity::{Arity, List as ArityList};

use super::{
	data::Memorize,
	operation::{BinOp, UnOp},
	{
		data::{
			AnyBinOp, AnyLoad, AnyStore, AnyUnOp, Backward, Br, BrIf, BrTable, Call, CallIndirect,
			Expression, Forward, Function, GetGlobal, GetLocal, If, MemoryGrow, MemorySize, Return,
			Select, SetGlobal, SetLocal, Statement, Value,
		},
		operation::{Load, Store},
	},
};

pub struct Transformer<'a> {
	// target state
	wasm: &'a Module,
	arity: &'a ArityList,
	name: usize,

	// translation state
	pending: Vec<Vec<Expression>>,
	stack: Vec<Expression>,
	last_stack: usize,
}

fn local_sum(list: &[Local]) -> u32 {
	list.iter().map(Local::count).sum()
}

fn is_else_stat(inst: &Instruction) -> bool {
	inst == &Instruction::Else
}

impl<'a> Transformer<'a> {
	pub fn new(wasm: &'a Module, arity: &'a ArityList, name: usize) -> Transformer<'a> {
		Transformer {
			wasm,
			arity,
			name,
			pending: Vec::new(),
			stack: Vec::new(),
			last_stack: 0,
		}
	}

	pub fn consume(mut self) -> Function {
		debug_assert!(self.name != usize::MAX, "Not an indexed value");

		let func = &self.wasm.code_section().unwrap().bodies()[self.name];
		let body = self.new_forward(&mut func.code().elements());

		Function {
			num_param: self.arity.in_arity[self.name].num_param,
			num_local: local_sum(func.locals()),
			num_stack: u32::try_from(self.last_stack).unwrap(),
			body,
		}
	}

	fn push_recall(&mut self, num: u32) {
		let len = self.stack.len();

		(len..len + num as usize)
			.map(Expression::Recall)
			.for_each(|v| self.stack.push(v));
	}

	fn push_block_result(&mut self, typ: BlockType) {
		if matches!(typ, BlockType::NoResult) {
			return;
		}

		self.push_recall(1);
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
			.filter(|v| !v.1.is_recalling(v.0))
		{
			let new = Expression::Recall(i);
			let mem = Memorize {
				var: i,
				value: std::mem::replace(v, new),
			};

			stat.push(Statement::Memorize(mem));
		}
	}

	// Pending expressions are put to sleep before entering
	// a control structure so that they are not lost.
	fn save_pending(&mut self) {
		self.pending.push(self.stack.clone());
	}

	fn load_pending(&mut self) {
		self.stack = self.pending.pop().unwrap();
	}

	fn gen_return(&mut self, stat: &mut Vec<Statement>) {
		let num = self.arity.in_arity[self.name].num_result as usize;
		let list = self.stack.split_off(self.stack.len() - num);

		self.gen_leak_pending(stat);
		stat.push(Statement::Return(Return { list }));
	}

	fn gen_call(&mut self, func: u32, stat: &mut Vec<Statement>) {
		let arity = self.arity.arity_of(func as usize);

		let param_list = self
			.stack
			.split_off(self.stack.len() - arity.num_param as usize);

		let len = u32::try_from(self.stack.len()).unwrap();
		let result = len..len + arity.num_result;

		self.push_recall(arity.num_result);
		self.gen_leak_pending(stat);
		stat.push(Statement::Call(Call {
			func,
			result,
			param_list,
		}));
	}

	fn gen_call_indirect(&mut self, typ: u32, table: u8, stat: &mut Vec<Statement>) {
		let types = self.wasm.type_section().unwrap().types();
		let arity = Arity::from_index(types, typ);

		let index = self.stack.pop().unwrap();
		let param_list = self
			.stack
			.split_off(self.stack.len() - arity.num_param as usize);

		let len = u32::try_from(self.stack.len()).unwrap();
		let result = len..len + arity.num_result;

		self.push_recall(arity.num_result);
		self.gen_leak_pending(stat);
		stat.push(Statement::CallIndirect(CallIndirect {
			table,
			index,
			result,
			param_list,
		}));
	}

	fn push_load(&mut self, op: Load, offset: u32) {
		let pointer = Box::new(self.stack.pop().unwrap());

		self.stack.push(Expression::AnyLoad(AnyLoad {
			op,
			offset,
			pointer,
		}));
	}

	fn gen_store(&mut self, op: Store, offset: u32, stat: &mut Vec<Statement>) {
		let value = self.stack.pop().unwrap();
		let pointer = self.stack.pop().unwrap();

		self.gen_leak_pending(stat);
		stat.push(Statement::AnyStore(AnyStore {
			op,
			offset,
			pointer,
			value,
		}));
	}

	fn push_constant(&mut self, value: Value) {
		self.stack.push(Expression::Value(value));
	}

	fn push_un_op(&mut self, op: UnOp) {
		let rhs = Box::new(self.stack.pop().unwrap());

		self.stack.push(Expression::AnyUnOp(AnyUnOp { op, rhs }));
	}

	fn push_bin_op(&mut self, op: BinOp) {
		let rhs = Box::new(self.stack.pop().unwrap());
		let lhs = Box::new(self.stack.pop().unwrap());

		self.stack
			.push(Expression::AnyBinOp(AnyBinOp { op, lhs, rhs }));
	}

	fn new_body(&mut self, list: &mut &[Instruction]) -> Vec<Statement> {
		use Instruction as Inst;

		let mut stat = Vec::new();

		loop {
			let inst = &list[0];

			*list = &list[1..];

			if let Ok(op) = UnOp::try_from(inst) {
				self.push_un_op(op);

				continue;
			} else if let Ok(op) = BinOp::try_from(inst) {
				self.push_bin_op(op);

				continue;
			}

			match inst {
				Inst::Nop => {}
				Inst::Unreachable => {
					self.gen_leak_pending(&mut stat);
					stat.push(Statement::Unreachable);
				}
				Inst::Block(t) => {
					self.gen_leak_pending(&mut stat);

					let data = self.new_forward(list);

					self.push_block_result(*t);
					stat.push(Statement::Forward(data));
				}
				Inst::Loop(t) => {
					self.gen_leak_pending(&mut stat);

					let data = self.new_backward(list);

					self.push_block_result(*t);
					stat.push(Statement::Backward(data));
				}
				Inst::If(t) => {
					let cond = self.stack.pop().unwrap();

					self.gen_leak_pending(&mut stat);

					let data = self.new_if(cond, list);

					self.push_block_result(*t);
					stat.push(Statement::If(data));
				}
				Inst::Else => {
					self.gen_leak_pending(&mut stat);

					break;
				}
				Inst::End => {
					if list.is_empty() && !self.stack.is_empty() {
						self.gen_return(&mut stat);
					} else {
						self.gen_leak_pending(&mut stat);
					}

					break;
				}
				Inst::Br(i) => {
					self.gen_leak_pending(&mut stat);
					stat.push(Statement::Br(Br { target: *i }));
				}
				Inst::BrIf(i) => {
					let cond = self.stack.pop().unwrap();

					self.gen_leak_pending(&mut stat);
					stat.push(Statement::BrIf(BrIf { cond, target: *i }));
				}
				Inst::BrTable(t) => {
					let cond = self.stack.pop().unwrap();

					self.gen_leak_pending(&mut stat);
					stat.push(Statement::BrTable(BrTable {
						cond,
						data: *t.clone(),
					}));
				}
				Inst::Return => {
					self.gen_return(&mut stat);
				}
				Inst::Call(i) => {
					self.gen_call(*i, &mut stat);
				}
				Inst::CallIndirect(i, t) => {
					self.gen_call_indirect(*i, *t, &mut stat);
				}
				Inst::Drop => {
					self.stack.pop().unwrap();
				}
				Inst::Select => {
					let cond = Box::new(self.stack.pop().unwrap());
					let b = Box::new(self.stack.pop().unwrap());
					let a = Box::new(self.stack.pop().unwrap());

					self.stack.push(Expression::Select(Select { cond, a, b }));
				}
				Inst::GetLocal(i) => {
					self.stack.push(Expression::GetLocal(GetLocal { var: *i }));
				}
				Inst::SetLocal(i) => {
					let value = self.stack.pop().unwrap();

					self.gen_leak_pending(&mut stat);
					stat.push(Statement::SetLocal(SetLocal { var: *i, value }));
				}
				Inst::TeeLocal(i) => {
					self.gen_leak_pending(&mut stat);

					let value = self.stack.last().unwrap().clone();

					stat.push(Statement::SetLocal(SetLocal { var: *i, value }));
				}
				Inst::GetGlobal(i) => {
					self.stack
						.push(Expression::GetGlobal(GetGlobal { var: *i }));
				}
				Inst::SetGlobal(i) => {
					let value = self.stack.pop().unwrap();

					stat.push(Statement::SetGlobal(SetGlobal { var: *i, value }));
				}
				Inst::I32Load(_, o) => self.push_load(Load::I32, *o),
				Inst::I64Load(_, o) => self.push_load(Load::I64, *o),
				Inst::F32Load(_, o) => self.push_load(Load::F32, *o),
				Inst::F64Load(_, o) => self.push_load(Load::F64, *o),
				Inst::I32Load8S(_, o) => self.push_load(Load::I32_I8, *o),
				Inst::I32Load8U(_, o) => self.push_load(Load::I32_U8, *o),
				Inst::I32Load16S(_, o) => self.push_load(Load::I32_I16, *o),
				Inst::I32Load16U(_, o) => self.push_load(Load::I32_U16, *o),
				Inst::I64Load8S(_, o) => self.push_load(Load::I64_I8, *o),
				Inst::I64Load8U(_, o) => self.push_load(Load::I64_U8, *o),
				Inst::I64Load16S(_, o) => self.push_load(Load::I64_I16, *o),
				Inst::I64Load16U(_, o) => self.push_load(Load::I64_U16, *o),
				Inst::I64Load32S(_, o) => self.push_load(Load::I64_I32, *o),
				Inst::I64Load32U(_, o) => self.push_load(Load::I64_U32, *o),
				Inst::I32Store(_, o) => self.gen_store(Store::I32, *o, &mut stat),
				Inst::I64Store(_, o) => self.gen_store(Store::I64, *o, &mut stat),
				Inst::F32Store(_, o) => self.gen_store(Store::F32, *o, &mut stat),
				Inst::F64Store(_, o) => self.gen_store(Store::F64, *o, &mut stat),
				Inst::I32Store8(_, o) => self.gen_store(Store::I32_N8, *o, &mut stat),
				Inst::I32Store16(_, o) => self.gen_store(Store::I32_N16, *o, &mut stat),
				Inst::I64Store8(_, o) => self.gen_store(Store::I64_N8, *o, &mut stat),
				Inst::I64Store16(_, o) => self.gen_store(Store::I64_N16, *o, &mut stat),
				Inst::I64Store32(_, o) => self.gen_store(Store::I64_N32, *o, &mut stat),
				Inst::CurrentMemory(i) => {
					self.stack
						.push(Expression::MemorySize(MemorySize { memory: *i }));
				}
				Inst::GrowMemory(i) => {
					let value = Box::new(self.stack.pop().unwrap());

					// `MemoryGrow` is an expression *but* it has side effects
					self.stack
						.push(Expression::MemoryGrow(MemoryGrow { memory: *i, value }));

					self.gen_leak_pending(&mut stat);
				}
				Inst::I32Const(v) => self.push_constant(Value::I32(*v)),
				Inst::I64Const(v) => self.push_constant(Value::I64(*v)),
				Inst::F32Const(v) => self.push_constant(Value::F32(f32::from_bits(*v))),
				Inst::F64Const(v) => self.push_constant(Value::F64(f64::from_bits(*v))),
				_ => unreachable!(),
			}
		}

		stat
	}

	fn new_stored_body(&mut self, list: &mut &[Instruction]) -> Vec<Statement> {
		self.save_pending();

		let body = self.new_body(list);

		self.load_pending();

		body
	}

	fn new_if(&mut self, cond: Expression, list: &mut &[Instruction]) -> If {
		let copied = list.clone();
		let truthy = self.new_stored_body(list);

		let last = copied.len() - list.len() - 1;
		let falsey = is_else_stat(&copied[last]).then(|| self.new_stored_body(list));

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
