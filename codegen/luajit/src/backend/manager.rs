use std::{
	collections::HashMap,
	io::{Result, Write},
	ops::Range,
};

use wasm_ast::node::BrTable;

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

#[derive(Default)]
pub struct Manager {
	table_map: HashMap<usize, usize>,
	label_list: Vec<usize>,
	num_label: usize,
	indentation: usize,
}

impl Manager {
	pub fn get_table_index(&self, table: &BrTable) -> usize {
		let id = table as *const _ as usize;

		self.table_map[&id]
	}

	pub fn set_table_map(&mut self, map: HashMap<usize, usize>) {
		self.table_map = map;
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

	pub fn indentation(&self) -> usize {
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

pub trait DriverNoContext {
	fn write(&self, w: &mut dyn Write) -> Result<()>;
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

pub fn write_ascending(prefix: &str, range: Range<usize>, w: &mut dyn Write) -> Result<()> {
	write_separated(range, |i, w| write!(w, "{prefix}_{i}"), w)
}
