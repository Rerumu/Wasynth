use crate::node::{
	BinOp, Block, Br, BrIf, BrTable, Call, CallIndirect, CmpOp, Expression, FuncData, GetGlobal,
	GetLocal, GetTemporary, If, LoadAt, MemoryGrow, MemorySize, MemoryCopy, MemoryFill, Select, SetGlobal, SetLocal,
	SetTemporary, Statement, StoreAt, Terminator, UnOp, Value,
};

pub trait Visitor {
	fn visit_select(&mut self, _: &Select) {}

	fn visit_get_temporary(&mut self, _: &GetTemporary) {}

	fn visit_get_local(&mut self, _: &GetLocal) {}

	fn visit_get_global(&mut self, _: &GetGlobal) {}

	fn visit_load_at(&mut self, _: &LoadAt) {}

	fn visit_memory_size(&mut self, _: &MemorySize) {}

	fn visit_value(&mut self, _: &Value) {}

	fn visit_un_op(&mut self, _: &UnOp) {}

	fn visit_bin_op(&mut self, _: &BinOp) {}

	fn visit_cmp_op(&mut self, _: &CmpOp) {}

	fn visit_expression(&mut self, _: &Expression) {}

	fn visit_unreachable(&mut self) {}

	fn visit_br(&mut self, _: &Br) {}

	fn visit_br_table(&mut self, _: &BrTable) {}

	fn visit_terminator(&mut self, _: &Terminator) {}

	fn visit_block(&mut self, _: &Block) {}

	fn visit_br_if(&mut self, _: &BrIf) {}

	fn visit_if(&mut self, _: &If) {}

	fn visit_call(&mut self, _: &Call) {}

	fn visit_call_indirect(&mut self, _: &CallIndirect) {}

	fn visit_set_temporary(&mut self, _: &SetTemporary) {}

	fn visit_set_local(&mut self, _: &SetLocal) {}

	fn visit_set_global(&mut self, _: &SetGlobal) {}

	fn visit_store_at(&mut self, _: &StoreAt) {}

	fn visit_memory_grow(&mut self, _: &MemoryGrow) {}

	fn visit_memory_copy(&mut self, _: &MemoryCopy) {}

	fn visit_memory_fill(&mut self, _: &MemoryFill) {}

	fn visit_statement(&mut self, _: &Statement) {}
}

pub trait Driver<T: Visitor> {
	fn accept(&self, visitor: &mut T);
}

impl<T: Visitor> Driver<T> for Select {
	fn accept(&self, visitor: &mut T) {
		self.condition().accept(visitor);
		self.on_true().accept(visitor);
		self.on_false().accept(visitor);

		visitor.visit_select(self);
	}
}

impl<T: Visitor> Driver<T> for GetTemporary {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_get_temporary(self);
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
		self.pointer().accept(visitor);

		visitor.visit_load_at(self);
	}
}

impl<T: Visitor> Driver<T> for MemorySize {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_memory_size(self);
	}
}

impl<T: Visitor> Driver<T> for MemoryCopy {
	fn accept(&self, visitor: &mut T) {
		self.size().accept(visitor);

		visitor.visit_memory_copy(self);
	}
}

impl<T: Visitor> Driver<T> for MemoryFill {
	fn accept(&self, visitor: &mut T) {
		self.value().accept(visitor);
		self.n().accept(visitor);

		visitor.visit_memory_fill(self);
	}
}

impl<T: Visitor> Driver<T> for Value {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_value(self);
	}
}

impl<T: Visitor> Driver<T> for UnOp {
	fn accept(&self, visitor: &mut T) {
		self.rhs().accept(visitor);

		visitor.visit_un_op(self);
	}
}

impl<T: Visitor> Driver<T> for BinOp {
	fn accept(&self, visitor: &mut T) {
		self.lhs().accept(visitor);
		self.rhs().accept(visitor);

		visitor.visit_bin_op(self);
	}
}

impl<T: Visitor> Driver<T> for CmpOp {
	fn accept(&self, visitor: &mut T) {
		self.lhs().accept(visitor);
		self.rhs().accept(visitor);

		visitor.visit_cmp_op(self);
	}
}

impl<T: Visitor> Driver<T> for Expression {
	fn accept(&self, visitor: &mut T) {
		match self {
			Self::Select(v) => v.accept(visitor),
			Self::GetTemporary(v) => v.accept(visitor),
			Self::GetLocal(v) => v.accept(visitor),
			Self::GetGlobal(v) => v.accept(visitor),
			Self::LoadAt(v) => v.accept(visitor),
			Self::MemorySize(v) => v.accept(visitor),
			Self::Value(v) => v.accept(visitor),
			Self::UnOp(v) => v.accept(visitor),
			Self::BinOp(v) => v.accept(visitor),
			Self::CmpOp(v) => v.accept(visitor),
		}

		visitor.visit_expression(self);
	}
}

impl<T: Visitor> Driver<T> for Br {
	fn accept(&self, visitor: &mut T) {
		visitor.visit_br(self);
	}
}

impl<T: Visitor> Driver<T> for BrTable {
	fn accept(&self, visitor: &mut T) {
		self.condition().accept(visitor);

		visitor.visit_br_table(self);
	}
}

impl<T: Visitor> Driver<T> for Terminator {
	fn accept(&self, visitor: &mut T) {
		match self {
			Self::Unreachable => visitor.visit_unreachable(),
			Self::Br(v) => v.accept(visitor),
			Self::BrTable(v) => v.accept(visitor),
		}

		visitor.visit_terminator(self);
	}
}

impl<T: Visitor> Driver<T> for Block {
	fn accept(&self, visitor: &mut T) {
		for v in self.code() {
			v.accept(visitor);
		}

		if let Some(v) = self.last() {
			v.accept(visitor);
		}

		visitor.visit_block(self);
	}
}

impl<T: Visitor> Driver<T> for BrIf {
	fn accept(&self, visitor: &mut T) {
		self.condition().accept(visitor);

		visitor.visit_br_if(self);
	}
}

impl<T: Visitor> Driver<T> for If {
	fn accept(&self, visitor: &mut T) {
		self.condition().accept(visitor);
		self.on_true().accept(visitor);

		if let Some(v) = self.on_false() {
			v.accept(visitor);
		}

		visitor.visit_if(self);
	}
}

impl<T: Visitor> Driver<T> for Call {
	fn accept(&self, visitor: &mut T) {
		for v in self.param_list() {
			v.accept(visitor);
		}

		visitor.visit_call(self);
	}
}

impl<T: Visitor> Driver<T> for CallIndirect {
	fn accept(&self, visitor: &mut T) {
		self.index().accept(visitor);

		for v in self.param_list() {
			v.accept(visitor);
		}

		visitor.visit_call_indirect(self);
	}
}

impl<T: Visitor> Driver<T> for SetTemporary {
	fn accept(&self, visitor: &mut T) {
		self.value().accept(visitor);

		visitor.visit_set_temporary(self);
	}
}

impl<T: Visitor> Driver<T> for SetLocal {
	fn accept(&self, visitor: &mut T) {
		self.value().accept(visitor);

		visitor.visit_set_local(self);
	}
}

impl<T: Visitor> Driver<T> for SetGlobal {
	fn accept(&self, visitor: &mut T) {
		self.value().accept(visitor);

		visitor.visit_set_global(self);
	}
}

impl<T: Visitor> Driver<T> for StoreAt {
	fn accept(&self, visitor: &mut T) {
		self.pointer().accept(visitor);
		self.value().accept(visitor);

		visitor.visit_store_at(self);
	}
}

impl<T: Visitor> Driver<T> for MemoryGrow {
	fn accept(&self, visitor: &mut T) {
		self.size().accept(visitor);

		visitor.visit_memory_grow(self);
	}
}

impl<T: Visitor> Driver<T> for Statement {
	fn accept(&self, visitor: &mut T) {
		match self {
			Self::Block(v) => v.accept(visitor),
			Self::BrIf(v) => v.accept(visitor),
			Self::If(v) => v.accept(visitor),
			Self::Call(v) => v.accept(visitor),
			Self::CallIndirect(v) => v.accept(visitor),
			Self::SetTemporary(v) => v.accept(visitor),
			Self::SetLocal(v) => v.accept(visitor),
			Self::SetGlobal(v) => v.accept(visitor),
			Self::StoreAt(v) => v.accept(visitor),
			Self::MemoryGrow(v) => v.accept(visitor),
			Self::MemoryCopy(v) => v.accept(visitor),
			Self::MemoryFill(v) => v.accept(visitor),
		}

		visitor.visit_statement(self);
	}
}

impl<T: Visitor> Driver<T> for FuncData {
	fn accept(&self, visitor: &mut T) {
		self.code().accept(visitor);
	}
}
