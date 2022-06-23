use std::collections::BTreeSet;

use wasm_ast::{
	node::{BinOp, CmpOp, FuncData, LoadAt, MemoryGrow, MemorySize, StoreAt, UnOp},
	visit::{Driver, Visitor},
};

use super::operator::{bin_symbol_of, cmp_symbol_of};

struct Visit {
	local_set: BTreeSet<(&'static str, &'static str)>,
	memory_set: BTreeSet<usize>,
}

impl Visitor for Visit {
	fn visit_load_at(&mut self, v: &LoadAt) {
		let name = v.load_type().as_name();

		self.memory_set.insert(0);
		self.local_set.insert(("load", name));
	}

	fn visit_store_at(&mut self, v: &StoreAt) {
		let name = v.store_type().as_name();

		self.memory_set.insert(0);
		self.local_set.insert(("store", name));
	}

	fn visit_un_op(&mut self, v: &UnOp) {
		let name = v.op_type().as_name();

		self.local_set.insert(name);
	}

	fn visit_bin_op(&mut self, v: &BinOp) {
		if bin_symbol_of(v.op_type()).is_some() {
			return;
		}

		let name = v.op_type().as_name();

		self.local_set.insert(name);
	}

	fn visit_cmp_op(&mut self, v: &CmpOp) {
		if cmp_symbol_of(v.op_type()).is_some() {
			return;
		}

		let name = v.op_type().as_name();

		self.local_set.insert(name);
	}

	fn visit_memory_size(&mut self, m: &MemorySize) {
		self.memory_set.insert(m.memory());
	}

	fn visit_memory_grow(&mut self, m: &MemoryGrow) {
		self.memory_set.insert(m.memory());
	}
}

pub fn visit(ast: &FuncData) -> (BTreeSet<(&'static str, &'static str)>, BTreeSet<usize>) {
	let mut visit = Visit {
		local_set: BTreeSet::new(),
		memory_set: BTreeSet::new(),
	};

	ast.accept(&mut visit);

	(visit.local_set, visit.memory_set)
}
