use std::{
	io::{Result, Write},
	ops::Range,
};

use wasm_ast::node::{CmpOp, Expression};

#[derive(Default)]
pub struct Manager {
	label_list: Vec<usize>,
	num_label: usize,
	pub num_param: usize,
}

impl Manager {
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

pub fn write_f32(number: f32, w: &mut dyn Write) -> Result<()> {
	let sign = if number.is_sign_negative() { "-" } else { "" };

	if number.is_infinite() {
		write!(w, "{sign}math.huge ")
	} else if number.is_nan() {
		write!(w, "{sign}0/0 ")
	} else {
		write!(w, "{number:e} ")
	}
}

pub fn write_f64(number: f64, w: &mut dyn Write) -> Result<()> {
	let sign = if number.is_sign_negative() { "-" } else { "" };

	if number.is_infinite() {
		write!(w, "{sign}math.huge ")
	} else if number.is_nan() {
		write!(w, "{sign}0/0 ")
	} else {
		write!(w, "{number:e} ")
	}
}

pub fn write_variable(var: usize, mng: &Manager, w: &mut dyn Write) -> Result<()> {
	if let Some(rem) = var.checked_sub(mng.num_param) {
		write!(w, "loc_{rem} ")
	} else {
		write!(w, "param_{var} ")
	}
}

pub fn write_bin_call(
	name: (&str, &str),
	lhs: &Expression,
	rhs: &Expression,
	mng: &mut Manager,
	w: &mut dyn Write,
) -> Result<()> {
	write!(w, "{}_{}(", name.0, name.1)?;
	lhs.write(mng, w)?;
	write!(w, ", ")?;
	rhs.write(mng, w)?;
	write!(w, ")")
}

pub fn write_cmp_op(cmp: &CmpOp, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	if let Some(op) = cmp.op.as_operator() {
		cmp.lhs.write(mng, w)?;
		write!(w, "{op} ")?;
		cmp.rhs.write(mng, w)
	} else {
		write_bin_call(cmp.op.as_name(), &cmp.lhs, &cmp.rhs, mng, w)
	}
}

pub fn write_condition(data: &Expression, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	if let Expression::CmpOp(node) = data {
		write_cmp_op(node, mng, w)
	} else {
		data.write(mng, w)?;
		write!(w, "~= 0 ")
	}
}
