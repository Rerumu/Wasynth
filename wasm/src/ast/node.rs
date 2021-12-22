use std::ops::Range;

use parity_wasm::elements::{BrTableData, ValueType};

use super::tag::{BinOp, CmpOp, Load, Store, UnOp};

#[derive(Clone)]
pub struct Recall {
	pub var: usize,
}

pub struct Select {
	pub cond: Box<Expression>,
	pub a: Box<Expression>,
	pub b: Box<Expression>,
}

pub struct GetLocal {
	pub var: u32,
}

pub struct GetGlobal {
	pub var: u32,
}

pub struct AnyLoad {
	pub op: Load,
	pub offset: u32,
	pub pointer: Box<Expression>,
}

pub struct MemorySize {
	pub memory: u8,
}

pub struct MemoryGrow {
	pub memory: u8,
	pub value: Box<Expression>,
}

#[derive(Clone, Copy)]
pub enum Value {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

pub struct AnyUnOp {
	pub op: UnOp,
	pub rhs: Box<Expression>,
}

pub struct AnyBinOp {
	pub op: BinOp,
	pub lhs: Box<Expression>,
	pub rhs: Box<Expression>,
}

pub struct AnyCmpOp {
	pub op: CmpOp,
	pub lhs: Box<Expression>,
	pub rhs: Box<Expression>,
}

pub enum Expression {
	Recall(Recall),
	Select(Select),
	GetLocal(GetLocal),
	GetGlobal(GetGlobal),
	AnyLoad(AnyLoad),
	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),
	Value(Value),
	AnyUnOp(AnyUnOp),
	AnyBinOp(AnyBinOp),
	AnyCmpOp(AnyCmpOp),
}

impl Expression {
	pub fn is_recalling(&self, wanted: usize) -> bool {
		match self {
			Expression::Recall(v) => v.var == wanted,
			_ => false,
		}
	}

	pub fn clone_recall(&self) -> Self {
		match self {
			Expression::Recall(v) => Expression::Recall(v.clone()),
			_ => unreachable!("clone_recall called on non-recall"),
		}
	}
}

pub struct Memorize {
	pub var: usize,
	pub value: Expression,
}

pub struct Forward {
	pub body: Vec<Statement>,
}

pub struct Backward {
	pub body: Vec<Statement>,
}

pub struct Else {
	pub body: Vec<Statement>,
}

pub struct If {
	pub cond: Expression,
	pub truthy: Vec<Statement>,
	pub falsey: Option<Else>,
}

pub struct Br {
	pub target: u32,
}

pub struct BrIf {
	pub cond: Expression,
	pub target: u32,
}

pub struct BrTable {
	pub cond: Expression,
	pub data: BrTableData,
}

pub struct Return {
	pub list: Vec<Expression>,
}

pub struct Call {
	pub func: u32,
	pub result: Range<u32>,
	pub param_list: Vec<Expression>,
}

pub struct CallIndirect {
	pub table: u8,
	pub index: Expression,
	pub result: Range<u32>,
	pub param_list: Vec<Expression>,
}

pub struct SetLocal {
	pub var: u32,
	pub value: Expression,
}

pub struct SetGlobal {
	pub var: u32,
	pub value: Expression,
}

pub struct AnyStore {
	pub op: Store,
	pub offset: u32,
	pub pointer: Expression,
	pub value: Expression,
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

pub struct Function {
	pub local_list: Vec<ValueType>,
	pub num_param: u32,
	pub num_stack: u32,
	pub body: Forward,
}
