use std::{
	io::{Result, Write},
	ops::Range,
};

use wasm_ast::node::{
	Block, Br, BrIf, BrTable, Call, CallIndirect, FuncData, If, LabelType, MemoryGrow, SetGlobal,
	SetLocal, SetTemporary, Statement, StoreAt, Terminator,
};
use wasmparser::ValType;

use crate::analyzer::br_target;

use super::manager::{
	write_ascending, write_condition, write_separated, write_variable, Driver, Manager,
};

impl Driver for Br {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if !self.align().is_aligned() {
			write_ascending("reg", self.align().new_range(), w)?;
			write!(w, " = ")?;
			write_ascending("reg", self.align().old_range(), w)?;
			write!(w, " ")?;
		}

		if self.target() == 0 {
			if let Some(Some(LabelType::Backward)) = mng.label_list().last() {
				write!(w, "continue ")
			} else {
				write!(w, "break ")
			}
		} else {
			let level = mng.label_list().len() - 1 - self.target();

			write!(w, "desired = {level} ")?;
			write!(w, "break ")
		}
	}
}

fn to_ordered_table<'a>(list: &'a [Br], default: &'a Br) -> Vec<&'a Br> {
	let mut data: Vec<_> = list.iter().chain(std::iter::once(default)).collect();

	data.sort_by_key(|v| v.target());
	data.dedup_by_key(|v| v.target());
	data
}

fn write_search_layer(
	range: Range<usize>,
	list: &[&Br],
	mng: &mut Manager,
	w: &mut dyn Write,
) -> Result<()> {
	if range.len() == 1 {
		return list[range.start].write(mng, w);
	}

	let center = range.start + range.len() / 2;
	let br = list[center];

	if range.start != center {
		write!(w, "if temp < {} then ", br.target())?;
		write_search_layer(range.start..center, list, mng, w)?;
		write!(w, "else")?;
	}

	if range.end != center + 1 {
		write!(w, "if temp > {} then ", br.target())?;
		write_search_layer(center + 1..range.end, list, mng, w)?;
		write!(w, "else")?;
	}

	write!(w, " ")?;
	br.write(mng, w)?;
	write!(w, "end ")
}

fn write_table_setup(table: &BrTable, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	let id = mng.get_table_index(table);

	write!(w, "if not br_map[{id}] then ")?;
	write!(w, "br_map[{id}] = (function() return {{[0] =")?;

	table
		.data()
		.iter()
		.try_for_each(|v| write!(w, "{},", v.target()))?;

	write!(w, "}} end)()")?;
	write!(w, "end ")?;

	write!(w, "local temp = br_map[{id}][")?;
	table.condition().write(mng, w)?;
	write!(w, "] or {} ", table.default().target())
}

impl Driver for BrTable {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if self.data().is_empty() {
			// Our condition should be pure so we probably don't need
			// to emit it in this case.
			return self.default().write(mng, w);
		}

		// `BrTable` is optimized by first mapping all indices to targets through
		// a Lua table; this reduces the size of the code generated as duplicate entries
		// don't need checking. Then, for speed, a binary search is done for the target
		// and the appropriate jump is performed.
		let list = to_ordered_table(self.data(), self.default());

		write_table_setup(self, mng, w)?;
		write_search_layer(0..list.len(), &list, mng, w)
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

fn write_br_gadget(
	label_list: &[Option<LabelType>],
	label: usize,
	w: &mut dyn Write,
) -> Result<()> {
	if label_list.len() == 1 || label_list.iter().all(Option::is_none) {
		return Ok(());
	}

	write!(w, "if desired then ")?;

	match label_list.last().unwrap() {
		Some(t) => {
			write!(w, "if desired == {label} then ")?;
			write!(w, "desired = nil ")?;

			if *t == LabelType::Backward {
				write!(w, "continue ")?;
			}

			write!(w, "else ")?;
			write!(w, "break ")?;
			write!(w, "end ")?;
		}
		None => {
			write!(w, "break ")?;
		}
	}

	write!(w, "end ")
}

impl Driver for Block {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let label = mng.push_label(self.label_type());

		write!(w, "while true do ")?;

		self.code().iter().try_for_each(|s| s.write(mng, w))?;

		match self.last() {
			Some(v) => v.write(mng, w)?,
			None => write!(w, "break ")?,
		}

		write!(w, "end ")?;

		write_br_gadget(mng.label_list(), label, w)?;
		mng.pop_label();

		Ok(())
	}
}

impl Driver for BrIf {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "if ")?;
		write_condition(self.condition(), mng, w)?;
		write!(w, "then ")?;
		self.target().write(mng, w)?;
		write!(w, "end ")
	}
}

impl Driver for If {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "if ")?;
		write_condition(self.condition(), mng, w)?;
		write!(w, "then ")?;

		self.on_true().write(mng, w)?;

		if let Some(v) = self.on_false() {
			write!(w, "else ")?;

			v.write(mng, w)?;
		}

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
		write_call_store(self.result(), w)?;

		write!(w, "FUNC_LIST[{}](", self.function())?;
		self.param_list().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for CallIndirect {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_call_store(self.result(), w)?;

		write!(w, "TABLE_LIST[{}].data[", self.table())?;
		self.index().write(mng, w)?;
		write!(w, "](")?;
		self.param_list().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for SetTemporary {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} = ", self.var())?;
		self.value().write(mng, w)
	}
}

impl Driver for SetLocal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_variable(self.var(), mng, w)?;
		write!(w, "= ")?;
		self.value().write(mng, w)
	}
}

impl Driver for SetGlobal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value = ", self.var())?;
		self.value().write(mng, w)
	}
}

impl Driver for StoreAt {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "store_{}(memory_at_0, ", self.store_type().as_name())?;
		self.pointer().write(mng, w)?;

		if self.offset() != 0 {
			write!(w, "+ {}", self.offset())?;
		}

		write!(w, ", ")?;
		self.value().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for MemoryGrow {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let result = self.result();
		let memory = self.memory();

		write!(w, "reg_{result} = rt.allocator.grow(memory_at_{memory}, ")?;
		self.size().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for Statement {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Block(s) => s.write(mng, w),
			Self::BrIf(s) => s.write(mng, w),
			Self::If(s) => s.write(mng, w),
			Self::Call(s) => s.write(mng, w),
			Self::CallIndirect(s) => s.write(mng, w),
			Self::SetTemporary(s) => s.write(mng, w),
			Self::SetLocal(s) => s.write(mng, w),
			Self::SetGlobal(s) => s.write(mng, w),
			Self::StoreAt(s) => s.write(mng, w),
			Self::MemoryGrow(s) => s.write(mng, w),
		}
	}
}

fn write_parameter_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	write!(w, "function(")?;
	write_ascending("param", 0..ast.num_param(), w)?;
	write!(w, ")")
}

fn write_variable_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	let mut total = 0;

	for data in ast.local_data().iter().filter(|v| v.0 != 0) {
		let range = total..total + usize::try_from(data.0).unwrap();
		let zero = if data.1 == ValType::I64 {
			"i64_ZERO "
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

	if ast.num_stack() != 0 {
		write!(w, "local ")?;
		write_ascending("reg", 0..ast.num_stack(), w)?;
		write!(w, " ")?;
	}

	Ok(())
}

impl Driver for FuncData {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let br_data = br_target::visit(self);

		write_parameter_list(self, w)?;
		write_variable_list(self, w)?;

		if br_data.1 {
			write!(w, "local desired ")?;
		}

		if !br_data.0.is_empty() {
			write!(w, "local br_map = {{}} ")?;
		}

		mng.set_table_map(br_data.0);
		mng.set_num_param(self.num_param());
		self.code().write(mng, w)?;

		if self.num_result() != 0 {
			write!(w, "return ")?;
			write_ascending("reg", 0..self.num_result(), w)?;
			write!(w, " ")?;
		}

		write!(w, "end ")
	}
}
