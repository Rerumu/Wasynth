use crate::backend::ast::data::{
	AnyBinOp, AnyLoad, AnyStore, AnyUnOp, Backward, Br, BrIf, BrTable, Call, CallIndirect,
	Expression, Forward, GetGlobal, GetLocal, If, Memorize, MemoryGrow, MemorySize, Return, Select,
	SetGlobal, SetLocal, Statement, Value,
};

pub trait Visitor {
	fn visit_recall(&mut self, _: usize) {}

	fn visit_select(&mut self, _: &Select) {}

	fn visit_get_local(&mut self, _: &GetLocal) {}

	fn visit_get_global(&mut self, _: &GetGlobal) {}

	fn visit_any_load(&mut self, _: &AnyLoad) {}

	fn visit_memory_size(&mut self, _: &MemorySize) {}

	fn visit_memory_grow(&mut self, _: &MemoryGrow) {}

	fn visit_value(&mut self, _: &Value) {}

	fn visit_any_unop(&mut self, _: &AnyUnOp) {}

	fn visit_any_binop(&mut self, _: &AnyBinOp) {}

	fn visit_expression(&mut self, _: &Expression) {}

	fn visit_unreachable(&mut self) {}

	fn visit_memorize(&mut self, _: &Memorize) {}

	fn visit_forward(&mut self, _: &Forward) {}

	fn visit_backward(&mut self, _: &Backward) {}

	fn visit_if(&mut self, _: &If) {}

	fn visit_br(&mut self, _: &Br) {}

	fn visit_br_if(&mut self, _: &BrIf) {}

	fn visit_br_table(&mut self, _: &BrTable) {}

	fn visit_return(&mut self, _: &Return) {}

	fn visit_call(&mut self, _: &Call) {}

	fn visit_call_indirect(&mut self, _: &CallIndirect) {}

	fn visit_set_local(&mut self, _: &SetLocal) {}

	fn visit_set_global(&mut self, _: &SetGlobal) {}

	fn visit_any_store(&mut self, _: &AnyStore) {}

	fn visit_statement(&mut self, _: &Statement) {}
}
