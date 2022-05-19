use std::{
	io::{Result, Write},
	ops::Range,
};

#[derive(PartialEq, Eq)]
pub enum Label {
	Forward,
	Backward,
	If,
}

#[derive(Default)]
pub struct Manager {
	label_list: Vec<Label>,
	pub num_param: usize,
}

impl Manager {
	pub fn label_list(&self) -> &[Label] {
		&self.label_list
	}

	pub fn push_label(&mut self, label: Label) -> usize {
		self.label_list.push(label);

		self.label_list.len() - 1
	}

	pub fn pop_label(&mut self) {
		self.label_list.pop().unwrap();
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

pub fn write_ascending(prefix: &str, range: Range<usize>, w: &mut dyn Write) -> Result<()> {
	write_separated(range, |i, w| write!(w, "{prefix}_{i}"), w)
}

pub fn write_variable(var: usize, mng: &Manager, w: &mut dyn Write) -> Result<()> {
	if let Some(rem) = var.checked_sub(mng.num_param) {
		write!(w, "loc_{rem} ")
	} else {
		write!(w, "param_{var} ")
	}
}
