use std::{
	io::{Result, Write},
	ops::Range,
};

use parity_wasm::elements::ValueType;
use wasm_ast::node::{
	Backward, Br, BrTable, Call, CallIndirect, Forward, FuncData, If, SetGlobal, SetLocal,
	SetTemporary, Statement, StoreAt, Terminator,
};

use super::manager::{
	write_ascending, write_condition, write_separated, write_variable, Driver, Label, Manager,
};

fn write_br_at(up: usize, mng: &Manager, w: &mut dyn Write) -> Result<()> {
	write!(w, "do ")?;

	if up == 0 {
		if let Some(&Label::Backward) = mng.label_list().last() {
			write!(w, "continue ")?;
		} else {
			write!(w, "break ")?;
		}
	} else {
		let level = mng.label_list().len() - 1 - up;

		write!(w, "desired = {level} ")?;
		write!(w, "break ")?;
	}

	write!(w, "end ")
}

impl Driver for Br {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_br_at(self.target, mng, w)
	}
}

impl Driver for BrTable {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "do ")?;
		write!(w, "local temp = {{")?;

		if !self.data.table.is_empty() {
			write!(w, "[0] =")?;

			for d in self.data.table.iter() {
				write!(w, "{d}, ")?;
			}
		}

		write!(w, "}} ")?;

		write!(w, "desired = temp[")?;
		self.cond.write(mng, w)?;
		write!(w, "] or {} ", self.data.default)?;
		write!(w, "break ")?;
		write!(w, "end ")
	}
}

impl Driver for Terminator {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Unreachable => write!(w, "error(\"out of code bounds\")"),
			Self::Br(s) => s.write(mng, w),
			Self::BrTable(s) => s.write(mng, w),
		}
	}
}

fn br_target(level: usize, in_loop: bool, w: &mut dyn Write) -> Result<()> {
	write!(w, "if desired then ")?;
	write!(w, "if desired == {level} then ")?;
	write!(w, "desired = nil ")?;

	if in_loop {
		write!(w, "continue ")?;
	}

	write!(w, "end ")?;
	write!(w, "break ")?;
	write!(w, "end ")
}

fn write_br_gadget(label_list: &[Label], rem: usize, w: &mut dyn Write) -> Result<()> {
	match label_list.last() {
		Some(Label::Forward | Label::If) => br_target(rem, false, w),
		Some(Label::Backward) => br_target(rem, true, w),
		None => Ok(()),
	}
}

impl Driver for Forward {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let rem = mng.push_label(Label::Forward);

		write!(w, "while true do ")?;

		self.code.iter().try_for_each(|s| s.write(mng, w))?;

		if let Some(v) = &self.last {
			v.write(mng, w)?;
		}

		write!(w, "break ")?;
		write!(w, "end ")?;

		mng.pop_label();
		write_br_gadget(mng.label_list(), rem, w)
	}
}

impl Driver for Backward {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let rem = mng.push_label(Label::Backward);

		write!(w, "while true do ")?;

		self.code.iter().try_for_each(|s| s.write(mng, w))?;

		if let Some(v) = &self.last {
			v.write(mng, w)?;
		}

		write!(w, "break ")?;
		write!(w, "end ")?;

		mng.pop_label();
		write_br_gadget(mng.label_list(), rem, w)
	}
}

impl Driver for If {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "while true do ")?;
		write!(w, "if ")?;
		write_condition(&self.cond, mng, w)?;
		write!(w, "then ")?;

		self.truthy.write(mng, w)?;

		if let Some(falsey) = &self.falsey {
			write!(w, "else ")?;

			falsey.write(mng, w)?;
		}

		write!(w, "end ")?;
		write!(w, "break ")?;
		write!(w, "end ")
	}
}

fn write_call_store(result: Range<usize>, w: &mut dyn Write) -> Result<()> {
	if result.is_empty() {
		return Ok(());
	}

	write_ascending("reg", result, w)?;
	write!(w, " = ")
}

impl Driver for Call {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_call_store(self.result.clone(), w)?;

		write!(w, "FUNC_LIST[{}](", self.func)?;
		self.param_list.as_slice().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for CallIndirect {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_call_store(self.result.clone(), w)?;

		write!(w, "TABLE_LIST[{}].data[", self.table)?;
		self.index.write(mng, w)?;
		write!(w, "](")?;
		self.param_list.as_slice().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for SetTemporary {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} = ", self.var)?;
		self.value.write(mng, w)
	}
}

impl Driver for SetLocal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_variable(self.var, mng, w)?;
		write!(w, "= ")?;
		self.value.write(mng, w)
	}
}

impl Driver for SetGlobal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value = ", self.var)?;
		self.value.write(mng, w)
	}
}

impl Driver for StoreAt {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "store_{}(memory_at_0, ", self.what.as_name())?;
		self.pointer.write(mng, w)?;

		if self.offset != 0 {
			write!(w, "+ {}", self.offset)?;
		}

		write!(w, ", ")?;
		self.value.write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for Statement {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Forward(s) => s.write(mng, w),
			Self::Backward(s) => s.write(mng, w),
			Self::If(s) => s.write(mng, w),
			Self::Call(s) => s.write(mng, w),
			Self::CallIndirect(s) => s.write(mng, w),
			Self::SetTemporary(s) => s.write(mng, w),
			Self::SetLocal(s) => s.write(mng, w),
			Self::SetGlobal(s) => s.write(mng, w),
			Self::StoreAt(s) => s.write(mng, w),
		}
	}
}

fn write_parameter_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	write!(w, "function(")?;
	write_ascending("param", 0..ast.num_param, w)?;
	write!(w, ")")
}

fn write_variable_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	let mut total = 0;

	for data in &ast.local_data {
		let range = total..total + usize::try_from(data.count()).unwrap();
		let zero = if data.value_type() == ValueType::I64 {
			"num_K_ZERO "
		} else {
			"0 "
		};

		total = range.end;

		write!(w, "local ")?;
		write_ascending("loc", range.clone(), w)?;
		write!(w, " = ")?;
		write_separated(range, |_, w| w.write_all(zero.as_bytes()), w)?;
		write!(w, " ")?;
	}

	if ast.num_stack != 0 {
		write!(w, "local ")?;
		write_ascending("reg", 0..ast.num_stack, w)?;
		write!(w, " ")?;
	}

	Ok(())
}

impl Driver for FuncData {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_parameter_list(self, w)?;
		write_variable_list(self, w)?;

		mng.num_param = self.num_param;
		self.code.write(mng, w)?;

		if self.num_result != 0 {
			write!(w, "return ")?;
			write_ascending("reg", 0..self.num_result, w)?;
			write!(w, " ")?;
		}

		write!(w, "end ")
	}
}
