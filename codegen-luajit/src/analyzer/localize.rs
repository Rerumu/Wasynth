use std::collections::BTreeSet;

use wasm_ast::{
	node::{BinOp, CmpOp, Intermediate, LoadAt, StoreAt, UnOp},
	visit::{Driver, Visitor},
};

struct Visit {
	result: BTreeSet<(&'static str, &'static str)>,
}

impl Visitor for Visit {
	fn visit_load_at(&mut self, v: &LoadAt) {
		let name = v.what.as_name();

		self.result.insert(("load", name));
	}

	fn visit_store_at(&mut self, v: &StoreAt) {
		let name = v.what.as_name();

		self.result.insert(("store", name));
	}

	fn visit_un_op(&mut self, v: &UnOp) {
		let name = v.op.as_name();

		self.result.insert(name);
	}

	fn visit_bin_op(&mut self, v: &BinOp) {
		if v.op.as_operator().is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}

	fn visit_cmp_op(&mut self, v: &CmpOp) {
		if v.op.as_operator().is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}
}

pub fn visit(ir: &Intermediate) -> BTreeSet<(&'static str, &'static str)> {
	let mut visit = Visit {
		result: BTreeSet::new(),
	};

	ir.accept(&mut visit);

	visit.result
}
