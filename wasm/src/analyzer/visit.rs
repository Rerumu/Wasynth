use crate::ast::node::{
	AnyBinOp, AnyLoad, AnyStore, AnyUnOp, Backward, Br, BrIf, BrTable, Call, CallIndirect, Else,
	Expression, Forward, Function, GetGlobal, GetLocal, If, Memorize, MemoryGrow, MemorySize,
	Recall, Return, Select, SetGlobal, SetLocal, Statement, Value,
};

pub trait Visitor {
	fn visit_recall(&mut self, _: &Recall) {}

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

	fn visit_else(&mut self, _: &Else) {}

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

pub trait Driver<T: Visitor> {
	fn accept(&self, visitor: &mut T);
}

impl<T: Visitor> Driver<T> for Recall {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_recall(self);
	}
}

impl<T: Visitor> Driver<T> for Select {
	fn accept(&self, visitor: &mut T) {
		self.cond.accept(visitor);
		self.a.accept(visitor);
		self.b.accept(visitor);

		visitor.visit_select(self);
	}
}

impl<T: Visitor> Driver<T> for GetLocal {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_get_local(self);
	}
}

impl<T: Visitor> Driver<T> for GetGlobal {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_get_global(self);
	}
}

impl<T: Visitor> Driver<T> for AnyLoad {
	fn accept(&self, visitor: &mut T) {
		self.pointer.accept(visitor);

		visitor.visit_any_load(self);
	}
}

impl<T: Visitor> Driver<T> for MemorySize {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_memory_size(self);
	}
}

impl<T: Visitor> Driver<T> for MemoryGrow {
	fn accept(&self, visitor: &mut T) {
		self.value.accept(visitor);

		visitor.visit_memory_grow(self);
	}
}

impl<T: Visitor> Driver<T> for Value {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_value(self);
	}
}

impl<T: Visitor> Driver<T> for AnyUnOp {
	fn accept(&self, visitor: &mut T) {
		self.rhs.accept(visitor);

		visitor.visit_any_unop(self);
	}
}

impl<T: Visitor> Driver<T> for AnyBinOp {
	fn accept(&self, visitor: &mut T) {
		self.lhs.accept(visitor);
		self.rhs.accept(visitor);

		visitor.visit_any_binop(self);
	}
}

impl<T: Visitor> Driver<T> for Expression {
	fn accept(&self, visitor: &mut T) {
		match self {
			Expression::Recall(v) => v.accept(visitor),
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

		visitor.visit_expression(self);
	}
}

impl<T: Visitor> Driver<T> for Memorize {
	fn accept(&self, visitor: &mut T) {
		self.value.accept(visitor);

		visitor.visit_memorize(self);
	}
}

impl<T: Visitor> Driver<T> for Forward {
	fn accept(&self, visitor: &mut T) {
		for v in &self.body {
			v.accept(visitor);
		}

		visitor.visit_forward(self);
	}
}

impl<T: Visitor> Driver<T> for Backward {
	fn accept(&self, visitor: &mut T) {
		for v in &self.body {
			v.accept(visitor);
		}

		visitor.visit_backward(self);
	}
}

impl<T: Visitor> Driver<T> for Else {
	fn accept(&self, visitor: &mut T) {
		for v in &self.body {
			v.accept(visitor);
		}

		visitor.visit_else(self);
	}
}

impl<T: Visitor> Driver<T> for If {
	fn accept(&self, visitor: &mut T) {
		self.cond.accept(visitor);

		for v in &self.truthy {
			v.accept(visitor);
		}

		if let Some(v) = &self.falsey {
			v.accept(visitor);
		}

		visitor.visit_if(self);
	}
}

impl<T: Visitor> Driver<T> for Br {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_br(self);
	}
}

impl<T: Visitor> Driver<T> for BrIf {
	fn accept(&self, visitor: &mut T) {
		self.cond.accept(visitor);

		visitor.visit_br_if(self);
	}
}

impl<T: Visitor> Driver<T> for BrTable {
	fn accept(&self, visitor: &mut T) {
		self.cond.accept(visitor);

		visitor.visit_br_table(self);
	}
}

impl<T: Visitor> Driver<T> for Return {
	fn accept(&self, visitor: &mut T) {
		for v in &self.list {
			v.accept(visitor);
		}

		visitor.visit_return(self);
	}
}

impl<T: Visitor> Driver<T> for Call {
	fn accept(&self, visitor: &mut T) {
		for v in &self.param_list {
			v.accept(visitor);
		}

		visitor.visit_call(self);
	}
}

impl<T: Visitor> Driver<T> for CallIndirect {
	fn accept(&self, visitor: &mut T) {
		self.index.accept(visitor);

		for v in &self.param_list {
			v.accept(visitor);
		}

		visitor.visit_call_indirect(self);
	}
}

impl<T: Visitor> Driver<T> for SetLocal {
	fn accept(&self, visitor: &mut T) {
		self.value.accept(visitor);

		visitor.visit_set_local(self);
	}
}

impl<T: Visitor> Driver<T> for SetGlobal {
	fn accept(&self, visitor: &mut T) {
		self.value.accept(visitor);

		visitor.visit_set_global(self);
	}
}

impl<T: Visitor> Driver<T> for AnyStore {
	fn accept(&self, visitor: &mut T) {
		self.pointer.accept(visitor);
		self.value.accept(visitor);

		visitor.visit_any_store(self);
	}
}

impl<T: Visitor> Driver<T> for Statement {
	fn accept(&self, visitor: &mut T) {
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

impl<T: Visitor> Driver<T> for Function {
	fn accept(&self, visitor: &mut T) {
		self.body.accept(visitor);
	}
}
