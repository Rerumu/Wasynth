use std::collections::BTreeSet;

use wasm_ast::{
	node::{AnyLoad, AnyStore, Function, MemoryGrow, MemorySize},
	visit::{Driver, Visitor},
};

struct Visit {
	result: BTreeSet<usize>,
}

impl Visitor for Visit {
	fn visit_any_store(&mut self, _: &AnyStore) {
		self.result.insert(0);
	}

	fn visit_any_load(&mut self, _: &AnyLoad) {
		self.result.insert(0);
	}

	fn visit_memory_size(&mut self, m: &MemorySize) {
		self.result.insert(m.memory);
	}

	fn visit_memory_grow(&mut self, m: &MemoryGrow) {
		self.result.insert(m.memory);
	}
}

pub fn visit(func: &Function) -> BTreeSet<usize> {
	let mut visit = Visit {
		result: BTreeSet::new(),
	};

	func.accept(&mut visit);

	visit.result
}
