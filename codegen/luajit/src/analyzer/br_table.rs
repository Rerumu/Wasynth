use std::collections::HashMap;

use wasm_ast::{
	node::{BrTable, FuncData},
	visit::{Driver, Visitor},
};

struct Visit {
	id_map: HashMap<usize, usize>,
}

impl Visitor for Visit {
	fn visit_br_table(&mut self, table: &BrTable) {
		if table.data().is_empty() {
			return;
		}

		let id = table as *const _ as usize;
		let len = self.id_map.len() + 1;

		self.id_map.insert(id, len);
	}
}

pub fn visit(ast: &FuncData) -> HashMap<usize, usize> {
	let mut visit = Visit {
		id_map: HashMap::new(),
	};

	ast.accept(&mut visit);

	visit.id_map
}
