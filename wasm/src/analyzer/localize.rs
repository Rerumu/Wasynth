use std::collections::BTreeSet;

use crate::ast::node::{AnyBinOp, AnyLoad, AnyStore, AnyUnOp, Function};

use super::visit::{Driver, Visitor};

struct Visit {
	result: BTreeSet<(&'static str, &'static str)>,
}

impl Visitor for Visit {
	fn visit_any_load(&mut self, v: &AnyLoad) {
		let name = v.op.as_name();

		self.result.insert(("load", name));
	}

	fn visit_any_store(&mut self, v: &AnyStore) {
		let name = v.op.as_name();

		self.result.insert(("store", name));
	}

	fn visit_any_unop(&mut self, v: &AnyUnOp) {
		if v.op.as_operator().is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}

	fn visit_any_binop(&mut self, v: &AnyBinOp) {
		if v.op.as_operator().is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}
}

pub fn visit(func: &Function) -> BTreeSet<(&'static str, &'static str)> {
	let mut visit = Visit {
		result: BTreeSet::new(),
	};

	func.accept(&mut visit);

	visit.result
}
