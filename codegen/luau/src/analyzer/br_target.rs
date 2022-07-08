use std::collections::HashMap;

use wasm_ast::{
	node::{Br, BrIf, BrTable, FuncData},
	visit::{Driver, Visitor},
};

struct Visit {
	br_map: HashMap<usize, usize>,
	has_branch: bool,
}

impl Visitor for Visit {
	fn visit_br(&mut self, _: &Br) {
		self.has_branch = true;
	}

	fn visit_br_if(&mut self, _: &BrIf) {
		self.has_branch = true;
	}

	fn visit_br_table(&mut self, table: &BrTable) {
		self.has_branch = true;

		if table.data().is_empty() {
			return;
		}

		let id = table as *const _ as usize;
		let len = self.br_map.len() + 1;

		self.br_map.insert(id, len);
	}
}

pub fn visit(ast: &FuncData) -> (HashMap<usize, usize>, bool) {
	let mut visit = Visit {
		br_map: HashMap::new(),
		has_branch: false,
	};

	ast.accept(&mut visit);

	(visit.br_map, visit.has_branch)
}
