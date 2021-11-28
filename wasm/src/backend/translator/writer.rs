use std::{
	io::{Result, Write},
	ops::Range,
};

use crate::backend::{
	ast::data::{
		AnyBinOp, AnyLoad, AnyStore, AnyUnOp, Backward, Br, BrIf, BrTable, Call, CallIndirect,
		Expression, Forward, Function, GetGlobal, GetLocal, If, Memorize, MemoryGrow, MemorySize,
		Return, Select, SetGlobal, SetLocal, Statement, Value,
	},
	edition::data::Edition,
	visitor::memory,
};

fn write_in_order(prefix: &'static str, len: u32, w: &mut dyn Write) -> Result<()> {
	if len == 0 {
		return Ok(());
	}

	write!(w, "{}_{}", prefix, 0)?;
	(1..len).try_for_each(|i| write!(w, ", {}_{}", prefix, i))
}

fn write_br_gadget(rem: usize, d: &mut Data, w: &mut dyn Write) -> Result<()> {
	match d.label_list.last() {
		Some(Label::Forward | Label::If) => d.edition.br_target(rem, false, w),
		Some(Label::Backward) => d.edition.br_target(rem, true, w),
		None => Ok(()),
	}
}

pub fn condense_jump_table(list: &[u32]) -> Vec<(usize, usize, u32)> {
	let mut result = Vec::new();
	let mut index = 0;

	while index < list.len() {
		let start = index;

		loop {
			index += 1;

			// if end of list or next value is not equal, break
			if index == list.len() || list[index - 1] != list[index] {
				break;
			}
		}

		result.push((start, index - 1, list[start]));
	}

	result
}

#[derive(PartialEq, Eq)]
enum Label {
	Forward,
	Backward,
	If,
}

pub struct Data<'a> {
	label_list: Vec<Label>,
	num_param: u32,
	edition: &'a dyn Edition,
}

impl<'a> Data<'a> {
	pub fn new(edition: &'a dyn Edition) -> Self {
		Self {
			label_list: Vec::new(),
			num_param: 0,
			edition,
		}
	}
}

impl Select {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		self.cond.output(d, w)?;
		write!(w, " ~= 0 and ")?;
		self.a.output(d, w)?;
		write!(w, " or ")?;
		self.b.output(d, w)?;
		write!(w, ")")
	}
}

impl GetLocal {
	fn write_variable(var: u32, d: &Data, w: &mut dyn Write) -> Result<()> {
		if let Some(rem) = var.checked_sub(d.num_param) {
			write!(w, "loc_{} ", rem)
		} else {
			write!(w, "param_{} ", var)
		}
	}

	fn output(&self, d: &Data, w: &mut dyn Write) -> Result<()> {
		Self::write_variable(self.var, d, w)
	}
}

impl GetGlobal {
	fn output(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value ", self.var)
	}
}

impl AnyLoad {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "load_{}(memory_at_0, ", self.op.as_name())?;
		self.pointer.output(d, w)?;
		write!(w, "+ {})", self.offset)
	}
}

impl MemorySize {
	fn output(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "rt.memory.size(memory_at_{})", self.memory)
	}
}

impl MemoryGrow {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "rt.memory.grow(memory_at_{}, ", self.memory)?;
		self.value.output(d, w)?;
		write!(w, ")")
	}
}

impl Value {
	fn write_f32(f: f32, w: &mut dyn Write) -> Result<()> {
		let sign = if f.is_sign_negative() { "-" } else { "" };

		if f.is_infinite() {
			write!(w, "{}math.huge", sign)
		} else if f.is_nan() {
			write!(w, "{}0/0", sign)
		} else {
			write!(w, "{:e}", f)
		}
	}

	fn write_f64(f: f64, w: &mut dyn Write) -> Result<()> {
		let sign = if f.is_sign_negative() { "-" } else { "" };

		if f.is_infinite() {
			write!(w, "{}math.huge", sign)
		} else if f.is_nan() {
			write!(w, "{}0/0", sign)
		} else {
			write!(w, "{:e}", f)
		}
	}

	fn output(&self, d: &Data, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::I32(i) => write!(w, "{} ", i),
			Self::I64(i) => write!(w, "{} ", d.edition.i64(*i)),
			Self::F32(f) => Self::write_f32(*f, w),
			Self::F64(f) => Self::write_f64(*f, w),
		}
	}
}

impl AnyUnOp {
	fn write_as_call(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		let (a, b) = self.op.as_name();

		write!(w, "{}_{}(", a, b)?;
		self.rhs.output(d, w)?;
		write!(w, ")")
	}

	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		if let Some(op) = self.op.as_operator() {
			write!(w, "{}", op)?;
			self.rhs.output(d, w)
		} else {
			self.write_as_call(d, w)
		}
	}
}

impl AnyBinOp {
	fn write_as_op(&self, op: &'static str, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		self.lhs.output(d, w)?;
		write!(w, "{} ", op)?;
		self.rhs.output(d, w)?;
		write!(w, ")")
	}

	fn write_as_call(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		let (a, b) = self.op.as_name();

		write!(w, "{}_{}(", a, b)?;
		self.lhs.output(d, w)?;
		write!(w, ", ")?;
		self.rhs.output(d, w)?;
		write!(w, ")")
	}

	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		if let Some(op) = self.op.as_operator() {
			self.write_as_op(op, d, w)
		} else {
			self.write_as_call(d, w)
		}
	}
}

impl Expression {
	fn write_list(list: &[Self], d: &mut Data, w: &mut dyn Write) -> Result<()> {
		list.iter().enumerate().try_for_each(|(i, v)| {
			if i != 0 {
				write!(w, ", ")?;
			}

			v.output(d, w)
		})
	}

	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Recall(i) => write!(w, "reg_{} ", i),
			Self::Select(s) => s.output(d, w),
			Self::GetLocal(g) => g.output(d, w),
			Self::GetGlobal(g) => g.output(w),
			Self::AnyLoad(a) => a.output(d, w),
			Self::MemorySize(m) => m.output(w),
			Self::MemoryGrow(m) => m.output(d, w),
			Self::Value(v) => v.output(d, w),
			Self::AnyUnOp(a) => a.output(d, w),
			Self::AnyBinOp(a) => a.output(d, w),
		}
	}

	fn to_buffer(&self, d: &mut Data) -> Result<String> {
		let mut buf = Vec::new();

		self.output(d, &mut buf)?;

		Ok(String::from_utf8(buf).unwrap())
	}
}

impl Memorize {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} = ", self.var)?;
		self.value.output(d, w)
	}
}

impl Forward {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		let rem = d.label_list.len();

		d.label_list.push(Label::Forward);
		d.edition.start_block(w)?;

		self.body.iter().try_for_each(|s| s.output(d, w))?;

		d.edition.end_block(rem, w)?;
		d.label_list.pop().unwrap();

		write_br_gadget(rem, d, w)
	}
}

impl Backward {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		let rem = d.label_list.len();

		d.label_list.push(Label::Backward);
		d.edition.start_loop(rem, w)?;

		self.body.iter().try_for_each(|s| s.output(d, w))?;

		d.edition.end_loop(w)?;
		d.label_list.pop().unwrap();

		write_br_gadget(rem, d, w)
	}
}

impl If {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		let rem = d.label_list.len();

		d.label_list.push(Label::If);

		let var = self.cond.to_buffer(d)?;

		d.edition.start_if(&var, w)?;

		self.truthy.iter().try_for_each(|s| s.output(d, w))?;

		if let Some(v) = &self.falsey {
			write!(w, "else ")?;

			v.iter().try_for_each(|s| s.output(d, w))?;
		}

		d.edition.end_if(rem, w)?;
		d.label_list.pop().unwrap();

		write_br_gadget(rem, d, w)
	}
}

impl Br {
	fn write_at(up: u32, d: &Data, w: &mut dyn Write) -> Result<()> {
		let up = up as usize;
		let level = d.label_list.len() - 1;
		let is_loop = d.label_list[level - up] == Label::Backward;

		d.edition.br_to_level(level, up, is_loop, w)
	}

	fn output(&self, d: &Data, w: &mut dyn Write) -> Result<()> {
		Self::write_at(self.target, d, w)
	}
}

impl BrIf {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "if ")?;
		self.cond.output(d, w)?;
		write!(w, "~= 0 then ")?;
		Br::write_at(self.target, d, w)?;
		write!(w, "end ")
	}
}

impl BrTable {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "local temp = ")?;
		self.cond.output(d, w)?;
		write!(w, " ")?;

		for (start, end, dest) in condense_jump_table(&self.data.table) {
			if start == end {
				write!(w, "if temp == {} then ", start)?;
			} else {
				write!(w, "if temp >= {} and temp <= {} then ", start, end)?;
			}

			Br::write_at(dest, d, w)?;
			write!(w, "else")?;
		}

		write!(w, " ")?;
		Br::write_at(self.data.default, d, w)?;
		write!(w, "end ")
	}
}

impl Return {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "do return ")?;

		self.list.iter().enumerate().try_for_each(|(i, v)| {
			if i > 0 {
				write!(w, ", ")?;
			}

			v.output(d, w)
		})?;

		write!(w, "end ")
	}
}

impl Call {
	fn write_result_list(range: Range<u32>, w: &mut dyn Write) -> Result<()> {
		if range.is_empty() {
			return Ok(());
		}

		range.clone().try_for_each(|i| {
			if i != range.start {
				write!(w, ", ")?;
			}

			write!(w, "reg_{}", i)
		})?;

		write!(w, " = ")
	}

	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		Self::write_result_list(self.result.clone(), w)?;

		write!(w, "FUNC_LIST[{}](", self.func)?;

		Expression::write_list(&self.param_list, d, w)?;

		write!(w, ")")
	}
}

impl CallIndirect {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		Call::write_result_list(self.result.clone(), w)?;

		write!(w, "TABLE_LIST[{}].data[", self.table)?;

		self.index.output(d, w)?;

		write!(w, "](")?;

		Expression::write_list(&self.param_list, d, w)?;

		write!(w, ")")
	}
}

impl SetLocal {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		GetLocal::write_variable(self.var, d, w)?;

		write!(w, "= ")?;
		self.value.output(d, w)
	}
}

impl SetGlobal {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value = ", self.var)?;
		self.value.output(d, w)
	}
}

impl AnyStore {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "store_{}(memory_at_0, ", self.op.as_name())?;
		self.pointer.output(d, w)?;
		write!(w, "+ {}, ", self.offset)?;
		self.value.output(d, w)?;
		write!(w, ")")
	}
}

impl Statement {
	fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		match self {
			Statement::Unreachable => write!(w, "error(\"out of code bounds\")"),
			Statement::Memorize(v) => v.output(d, w),
			Statement::Forward(v) => v.output(d, w),
			Statement::Backward(v) => v.output(d, w),
			Statement::If(v) => v.output(d, w),
			Statement::Br(v) => v.output(d, w),
			Statement::BrIf(v) => v.output(d, w),
			Statement::BrTable(v) => v.output(d, w),
			Statement::Return(v) => v.output(d, w),
			Statement::Call(v) => v.output(d, w),
			Statement::CallIndirect(v) => v.output(d, w),
			Statement::SetLocal(v) => v.output(d, w),
			Statement::SetGlobal(v) => v.output(d, w),
			Statement::AnyStore(v) => v.output(d, w),
		}
	}
}

impl Function {
	fn write_variable_list(&self, w: &mut dyn Write) -> Result<()> {
		if self.num_local != 0 {
			let list = vec!["0"; self.num_local as usize].join(", ");

			write!(w, "local ")?;
			write_in_order("loc", self.num_local, w)?;
			write!(w, " = {} ", list)?;
		}

		if self.num_stack != 0 {
			write!(w, "local ")?;
			write_in_order("reg", self.num_stack, w)?;
			write!(w, " ")?;
		}

		Ok(())
	}

	pub fn output(&self, d: &mut Data, w: &mut dyn Write) -> Result<()> {
		write!(w, "function(")?;

		write_in_order("param", self.num_param, w)?;

		write!(w, ")")?;

		d.num_param = self.num_param;

		for v in memory::visit(self) {
			write!(w, "local memory_at_{0} = MEMORY_LIST[{0}]", v)?;
		}

		self.write_variable_list(w)?;

		self.body.output(d, w)?;

		write!(w, "end ")
	}
}
