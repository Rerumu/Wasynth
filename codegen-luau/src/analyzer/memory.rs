use std::collections::BTreeSet;

use wasm_ast::{
	node::{Intermediate, LoadAt, MemoryGrow, MemorySize, StoreAt},
	visit::{Driver, Visitor},
};

struct Visit {
	result: BTreeSet<usize>,
}

impl Visitor for Visit {
	fn visit_store_at(&mut self, _: &StoreAt) {
		self.result.insert(0);
	}

	fn visit_load_at(&mut self, _: &LoadAt) {
		self.result.insert(0);
	}

	fn visit_memory_size(&mut self, m: &MemorySize) {
		self.result.insert(m.memory);
	}

	fn visit_memory_grow(&mut self, m: &MemoryGrow) {
		self.result.insert(m.memory);
	}
}

pub fn visit(ir: &Intermediate) -> BTreeSet<usize> {
	let mut visit = Visit {
		result: BTreeSet::new(),
	};

	ir.accept(&mut visit);

	visit.result
}
