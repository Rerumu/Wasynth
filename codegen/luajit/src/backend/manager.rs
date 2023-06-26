use std::{
	collections::HashMap,
	io::{Result, Write},
};

use wasm_ast::node::{BrTable, FuncData};

use crate::analyzer::{br_table, localize};

#[macro_export]
macro_rules! indentation {
	($mng:tt, $w:tt) => {{
		let mut iter = 0..$mng.indentation();

		iter.try_for_each(|_| write!($w, "\t"))
	}};
}

#[macro_export]
macro_rules! indented {
	($mng:tt, $w:tt, $($args:tt)*) => {{
		indentation!($mng, $w)?;
		write!($w, $($args)*)
	}};
}

#[macro_export]
macro_rules! line {
	($mng:tt, $w:tt, $($args:tt)*) => {{
		indentation!($mng, $w)?;
		writeln!($w, $($args)*)
	}};
}

fn get_pinned_registers(
	upvalues: usize,
	params: usize,
	locals: usize,
	temporaries: usize,
) -> (usize, usize) {
	const MAX_LOCAL_COUNT: usize = 180;

	let available = MAX_LOCAL_COUNT
		.saturating_sub(upvalues)
		.saturating_sub(params);

	let temporaries = available.min(temporaries);
	let locals = available.saturating_sub(temporaries).min(locals);

	(params + locals, temporaries)
}

pub struct Manager {
	table_map: HashMap<usize, usize>,
	num_local: usize,
	num_temp: usize,
	num_label: usize,
	label_list: Vec<usize>,
	indentation: usize,
}

impl Manager {
	pub fn empty() -> Self {
		Self {
			table_map: HashMap::new(),
			num_local: 0,
			num_temp: usize::MAX,
			num_label: 0,
			label_list: Vec::new(),
			indentation: 0,
		}
	}

	pub fn function(ast: &FuncData) -> Self {
		let (upvalues, memories) = localize::visit(ast);
		let table_map = br_table::visit(ast);
		let (num_local, num_temp) = get_pinned_registers(
			upvalues.len() + memories.len(),
			ast.num_param(),
			ast.local_data().len(),
			ast.num_stack(),
		);

		Self {
			table_map,
			num_local,
			num_temp,
			num_label: 0,
			label_list: Vec::new(),
			indentation: 0,
		}
	}

	pub fn get_table_index(&self, table: &BrTable) -> usize {
		let id = table as *const _ as usize;

		self.table_map[&id]
	}

	pub fn has_table(&self) -> bool {
		!self.table_map.is_empty()
	}

	pub const fn num_local(&self) -> usize {
		self.num_local
	}

	pub const fn num_temp(&self) -> usize {
		self.num_temp
	}

	pub fn label_list(&self) -> &[usize] {
		&self.label_list
	}

	pub fn push_label(&mut self) -> usize {
		self.label_list.push(self.num_label);
		self.num_label += 1;

		self.num_label - 1
	}

	pub fn pop_label(&mut self) {
		self.label_list.pop().unwrap();
	}

	pub const fn indentation(&self) -> usize {
		self.indentation
	}

	pub fn indent(&mut self) {
		self.indentation += 1;
	}

	pub fn dedent(&mut self) {
		self.indentation -= 1;
	}
}

pub trait Driver {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()>;
}

pub fn write_separated<I, T, M>(mut iter: I, mut func: M, w: &mut dyn Write) -> Result<()>
where
	M: FnMut(T, &mut dyn Write) -> Result<()>,
	I: Iterator<Item = T>,
{
	match iter.next() {
		Some(first) => func(first, w)?,
		None => return Ok(()),
	}

	iter.try_for_each(|v| {
		write!(w, ", ")?;
		func(v, w)
	})
}
