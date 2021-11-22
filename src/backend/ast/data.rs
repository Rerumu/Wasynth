use std::ops::Range;

use parity_wasm::elements::BrTableData;

use crate::backend::visitor::data::Visitor;

use super::operation::{BinOp, Load, Store, UnOp};

#[derive(Clone)]
pub struct Select {
	pub cond: Box<Expression>,
	pub a: Box<Expression>,
	pub b: Box<Expression>,
}

impl Select {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_select(self);

		self.cond.accept(visitor);
		self.a.accept(visitor);
		self.b.accept(visitor);
	}
}

#[derive(Clone)]
pub struct GetLocal {
	pub var: u32,
}

impl GetLocal {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_get_local(self);
	}
}

#[derive(Clone)]
pub struct GetGlobal {
	pub var: u32,
}

impl GetGlobal {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_get_global(self);
	}
}

#[derive(Clone)]
pub struct AnyLoad {
	pub op: Load,
	pub offset: u32,
	pub pointer: Box<Expression>,
}

impl AnyLoad {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_any_load(self);

		self.pointer.accept(visitor);
	}
}

#[derive(Clone)]
pub struct MemorySize {
	pub memory: u8,
}

impl MemorySize {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_memory_size(self);
	}
}

#[derive(Clone)]
pub struct MemoryGrow {
	pub memory: u8,
	pub value: Box<Expression>,
}

impl MemoryGrow {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_memory_grow(self);

		self.value.accept(visitor);
	}
}

#[derive(Clone, Copy)]
pub enum Value {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

impl Value {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_value(self);
	}
}

#[derive(Clone)]
pub struct AnyUnOp {
	pub op: UnOp,
	pub rhs: Box<Expression>,
}

impl AnyUnOp {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_any_unop(self);

		self.rhs.accept(visitor);
	}
}

#[derive(Clone)]
pub struct AnyBinOp {
	pub op: BinOp,
	pub lhs: Box<Expression>,
	pub rhs: Box<Expression>,
}

impl AnyBinOp {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_any_binop(self);

		self.lhs.accept(visitor);
		self.rhs.accept(visitor);
	}
}

#[derive(Clone)]
pub enum Expression {
	Recall(usize),
	Select(Select),
	GetLocal(GetLocal),
	GetGlobal(GetGlobal),
	AnyLoad(AnyLoad),
	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),
	Value(Value),
	AnyUnOp(AnyUnOp),
	AnyBinOp(AnyBinOp),
}

impl Expression {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_expression(self);

		match self {
			Expression::Recall(v) => visitor.visit_recall(*v),
			Expression::Select(v) => v.accept(visitor),
			Expression::GetLocal(v) => v.accept(visitor),
			Expression::GetGlobal(v) => v.accept(visitor),
			Expression::AnyLoad(v) => v.accept(visitor),
			Expression::MemorySize(v) => v.accept(visitor),
			Expression::MemoryGrow(v) => v.accept(visitor),
			Expression::Value(v) => v.accept(visitor),
			Expression::AnyUnOp(v) => v.accept(visitor),
			Expression::AnyBinOp(v) => v.accept(visitor),
		}
	}

	pub fn is_recalling(&self, wanted: usize) -> bool {
		match *self {
			Expression::Recall(v) => v == wanted,
			_ => false,
		}
	}
}

pub struct Memorize {
	pub var: usize,
	pub value: Expression,
}

impl Memorize {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_memorize(self);

		self.value.accept(visitor);
	}
}

pub struct Forward {
	pub body: Vec<Statement>,
}

impl Forward {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_forward(self);

		for v in &self.body {
			v.accept(visitor);
		}
	}
}

pub struct Backward {
	pub body: Vec<Statement>,
}

impl Backward {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_backward(self);

		for v in &self.body {
			v.accept(visitor);
		}
	}
}

pub struct If {
	pub cond: Expression,
	pub body: Vec<Statement>,
	pub other: Option<Vec<Statement>>,
}

impl If {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_if(self);

		self.cond.accept(visitor);

		for v in &self.body {
			v.accept(visitor);
		}

		if let Some(v) = &self.other {
			for v in v {
				v.accept(visitor);
			}
		}
	}
}

pub struct Br {
	pub target: u32,
}

impl Br {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_br(self);
	}
}

pub struct BrIf {
	pub cond: Expression,
	pub target: u32,
}

impl BrIf {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_br_if(self);

		self.cond.accept(visitor);
	}
}

pub struct BrTable {
	pub cond: Expression,
	pub data: BrTableData,
}

impl BrTable {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_br_table(self);

		self.cond.accept(visitor);
	}
}

pub struct Return {
	pub list: Vec<Expression>,
}

impl Return {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_return(self);

		for v in &self.list {
			v.accept(visitor);
		}
	}
}

pub struct Call {
	pub func: u32,
	pub result: Range<u32>,
	pub param_list: Vec<Expression>,
}

impl Call {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_call(self);

		for v in &self.param_list {
			v.accept(visitor);
		}
	}
}

pub struct CallIndirect {
	pub table: u8,
	pub index: Expression,
	pub result: Range<u32>,
	pub param_list: Vec<Expression>,
}

impl CallIndirect {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_call_indirect(self);

		self.index.accept(visitor);

		for v in &self.param_list {
			v.accept(visitor);
		}
	}
}

pub struct SetLocal {
	pub var: u32,
	pub value: Expression,
}

impl SetLocal {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_set_local(self);

		self.value.accept(visitor);
	}
}

pub struct SetGlobal {
	pub var: u32,
	pub value: Expression,
}

impl SetGlobal {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_set_global(self);

		self.value.accept(visitor);
	}
}

pub struct AnyStore {
	pub op: Store,
	pub offset: u32,
	pub pointer: Expression,
	pub value: Expression,
}

impl AnyStore {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		visitor.visit_any_store(self);

		self.pointer.accept(visitor);
		self.value.accept(visitor);
	}
}

pub enum Statement {
	Unreachable,
	Memorize(Memorize),
	Forward(Forward),
	Backward(Backward),
	If(If),
	Br(Br),
	BrIf(BrIf),
	BrTable(BrTable),
	Return(Return),
	Call(Call),
	CallIndirect(CallIndirect),
	SetLocal(SetLocal),
	SetGlobal(SetGlobal),
	AnyStore(AnyStore),
}

impl Statement {
	fn accept<V: Visitor>(&self, visitor: &mut V) {
		match self {
			Statement::Unreachable => visitor.visit_unreachable(),
			Statement::Memorize(v) => v.accept(visitor),
			Statement::Forward(v) => v.accept(visitor),
			Statement::Backward(v) => v.accept(visitor),
			Statement::If(v) => v.accept(visitor),
			Statement::Br(v) => v.accept(visitor),
			Statement::BrIf(v) => v.accept(visitor),
			Statement::BrTable(v) => v.accept(visitor),
			Statement::Return(v) => v.accept(visitor),
			Statement::Call(v) => v.accept(visitor),
			Statement::CallIndirect(v) => v.accept(visitor),
			Statement::SetLocal(v) => v.accept(visitor),
			Statement::SetGlobal(v) => v.accept(visitor),
			Statement::AnyStore(v) => v.accept(visitor),
		}
	}
}

pub struct Function {
	pub num_param: u32,
	pub num_local: u32,
	pub num_stack: u32,
	pub body: Vec<Statement>,
}

impl Function {
	pub fn accept<V: Visitor>(&self, visitor: &mut V) {
		for v in &self.body {
			v.accept(visitor);
		}
	}
}
