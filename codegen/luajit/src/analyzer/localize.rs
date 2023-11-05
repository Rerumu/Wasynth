use std::collections::BTreeSet;

use wasm_ast::{
	node::{
		BinOp, CmpOp, FuncData, LoadAt, MemoryCopy, MemoryFill, MemoryGrow, MemorySize, StoreAt,
		UnOp,
	},
	visit::{Driver, Visitor},
};

use super::into_string::{IntoName, IntoNameTuple, TryIntoSymbol};

struct Visit {
	local_set: BTreeSet<(&'static str, &'static str)>,
	memory_set: BTreeSet<usize>,
}

impl Visitor for Visit {
	fn visit_load_at(&mut self, v: &LoadAt) {
		let name = v.load_type().into_name();

		self.memory_set.insert(v.memory());
		self.local_set.insert(("load", name));
	}

	fn visit_store_at(&mut self, v: &StoreAt) {
		let name = v.store_type().into_name();

		self.memory_set.insert(v.memory());
		self.local_set.insert(("store", name));
	}

	fn visit_un_op(&mut self, v: &UnOp) {
		let name = v.op_type().into_name_tuple();

		self.local_set.insert(name);
	}

	fn visit_bin_op(&mut self, v: &BinOp) {
		if v.op_type().try_into_symbol().is_some() {
			return;
		}

		let name = v.op_type().into_name_tuple();

		self.local_set.insert(name);
	}

	fn visit_cmp_op(&mut self, v: &CmpOp) {
		if v.op_type().try_into_symbol().is_some() {
			return;
		}

		let name = v.op_type().into_name_tuple();

		self.local_set.insert(name);
	}

	fn visit_memory_size(&mut self, m: &MemorySize) {
		self.memory_set.insert(m.memory());
	}

	fn visit_memory_grow(&mut self, m: &MemoryGrow) {
		self.memory_set.insert(m.memory());
	}

	fn visit_memory_copy(&mut self, m: &MemoryCopy) {
		self.memory_set.insert(m.destination().memory());
		self.memory_set.insert(m.source().memory());
	}

	fn visit_memory_fill(&mut self, m: &MemoryFill) {
		self.memory_set.insert(m.destination().memory());
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
