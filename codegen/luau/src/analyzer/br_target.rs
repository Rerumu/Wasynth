use std::collections::HashMap;

use wasm_ast::{
	node::{Br, BrIf, BrTable, FuncData},
	visit::{Driver, Visitor},
};

struct Visit {
	br_map: HashMap<usize, usize>,
	has_branch: bool,
}

impl Visit {
	fn set_branch(&mut self, br: &Br) {
		if br.target() != 0 {
			self.has_branch = true;
		}
	}
}

impl Visitor for Visit {
	fn visit_br(&mut self, stat: &Br) {
		self.set_branch(stat);
	}

	fn visit_br_if(&mut self, stat: &BrIf) {
		self.set_branch(stat.target());
	}

	fn visit_br_table(&mut self, table: &BrTable) {
		self.set_branch(table.default());

		if table.data().is_empty() {
			return;
		}

		for target in table.data() {
			self.set_branch(target);
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
