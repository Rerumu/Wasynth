use crate::node::{
	Backward, BinOp, Br, BrIf, BrTable, Call, CallIndirect, CmpOp, Else, Expression, Forward,
	GetGlobal, GetLocal, If, Intermediate, LoadAt, Memorize, MemoryGrow, MemorySize, Recall,
	Return, Select, SetGlobal, SetLocal, Statement, StoreAt, UnOp, Value,
};

pub trait Visitor {
	fn visit_recall(&mut self, _: &Recall) {}

	fn visit_select(&mut self, _: &Select) {}

	fn visit_get_local(&mut self, _: &GetLocal) {}

	fn visit_get_global(&mut self, _: &GetGlobal) {}

	fn visit_load_at(&mut self, _: &LoadAt) {}

	fn visit_memory_size(&mut self, _: &MemorySize) {}

	fn visit_memory_grow(&mut self, _: &MemoryGrow) {}

	fn visit_value(&mut self, _: &Value) {}

	fn visit_un_op(&mut self, _: &UnOp) {}

	fn visit_bin_op(&mut self, _: &BinOp) {}

	fn visit_cmp_op(&mut self, _: &CmpOp) {}

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

	fn visit_store_at(&mut self, _: &StoreAt) {}

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

impl<T: Visitor> Driver<T> for LoadAt {
	fn accept(&self, visitor: &mut T) {
		self.pointer.accept(visitor);

		visitor.visit_load_at(self);
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

impl<T: Visitor> Driver<T> for UnOp {
	fn accept(&self, visitor: &mut T) {
		self.rhs.accept(visitor);

		visitor.visit_un_op(self);
	}
}

impl<T: Visitor> Driver<T> for BinOp {
	fn accept(&self, visitor: &mut T) {
		self.lhs.accept(visitor);
		self.rhs.accept(visitor);

		visitor.visit_bin_op(self);
	}
}

impl<T: Visitor> Driver<T> for CmpOp {
	fn accept(&self, visitor: &mut T) {
		self.lhs.accept(visitor);
		self.rhs.accept(visitor);

		visitor.visit_cmp_op(self);
	}
}

impl<T: Visitor> Driver<T> for Expression {
	fn accept(&self, visitor: &mut T) {
		match self {
			Self::Recall(v) => v.accept(visitor),
			Self::Select(v) => v.accept(visitor),
			Self::GetLocal(v) => v.accept(visitor),
			Self::GetGlobal(v) => v.accept(visitor),
			Self::LoadAt(v) => v.accept(visitor),
			Self::MemorySize(v) => v.accept(visitor),
			Self::MemoryGrow(v) => v.accept(visitor),
			Self::Value(v) => v.accept(visitor),
			Self::UnOp(v) => v.accept(visitor),
			Self::BinOp(v) => v.accept(visitor),
			Self::CmpOp(v) => v.accept(visitor),
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

impl<T: Visitor> Driver<T> for StoreAt {
	fn accept(&self, visitor: &mut T) {
		self.pointer.accept(visitor);
		self.value.accept(visitor);

		visitor.visit_store_at(self);
	}
}

impl<T: Visitor> Driver<T> for Statement {
	fn accept(&self, visitor: &mut T) {
		match self {
			Self::Unreachable => visitor.visit_unreachable(),
			Self::Memorize(v) => v.accept(visitor),
			Self::Forward(v) => v.accept(visitor),
			Self::Backward(v) => v.accept(visitor),
			Self::If(v) => v.accept(visitor),
			Self::Br(v) => v.accept(visitor),
			Self::BrIf(v) => v.accept(visitor),
			Self::BrTable(v) => v.accept(visitor),
			Self::Return(v) => v.accept(visitor),
			Self::Call(v) => v.accept(visitor),
			Self::CallIndirect(v) => v.accept(visitor),
			Self::SetLocal(v) => v.accept(visitor),
			Self::SetGlobal(v) => v.accept(visitor),
			Self::StoreAt(v) => v.accept(visitor),
		}
	}
}

impl<T: Visitor> Driver<T> for Intermediate {
	fn accept(&self, visitor: &mut T) {
		self.code.accept(visitor);
	}
}
