use std::collections::BTreeSet;

use wasm_ast::{
	node::{BinOp, CmpOp, FuncData, LoadAt, StoreAt, UnOp, Value},
	visit::{Driver, Visitor},
};

use super::operator::{bin_symbol_of, cmp_symbol_of};

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

	fn visit_value(&mut self, v: &Value) {
		let name = match v {
			Value::I64(0) => "K_ZERO",
			Value::I64(1) => "K_ONE",
			Value::I64(_) => "from_u32",
			_ => return,
		};

		self.result.insert(("i64", name));
	}

	fn visit_un_op(&mut self, v: &UnOp) {
		let name = v.op.as_name();

		self.result.insert(name);
	}

	fn visit_bin_op(&mut self, v: &BinOp) {
		if bin_symbol_of(v.op).is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}

	fn visit_cmp_op(&mut self, v: &CmpOp) {
		if cmp_symbol_of(v.op).is_some() {
			return;
		}

		let name = v.op.as_name();

		self.result.insert(name);
	}
}

pub fn visit(ast: &FuncData) -> BTreeSet<(&'static str, &'static str)> {
	let mut visit = Visit {
		result: BTreeSet::new(),
	};

	ast.accept(&mut visit);

	visit.result
}
