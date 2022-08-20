use std::{
	io::{Result, Write},
	ops::Range,
};

use wasm_ast::node::{
	Block, Br, BrIf, BrTable, Call, CallIndirect, FuncData, If, LabelType, MemoryGrow, SetGlobal,
	SetLocal, SetTemporary, Statement, StoreAt, Terminator,
};
use wasmparser::ValType;

use crate::analyzer::br_table;

use super::{
	expression::Condition,
	manager::{write_ascending, write_separated, Driver, DriverNoContext, Manager},
};

impl Driver for Br {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let level = *mng.label_list().iter().nth_back(self.target()).unwrap();

		if !self.align().is_aligned() {
			write_ascending("reg", self.align().new_range(), w)?;
			write!(w, " = ")?;
			write_ascending("reg", self.align().old_range(), w)?;
			write!(w, " ")?;
		}

		write!(w, "goto continue_at_{level} ")
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

	write!(w, "temp = br_map[{id}][")?;
	table.condition().write(w)?;
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

fn write_inner_block(block: &Block, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	block.code().iter().try_for_each(|s| s.write(mng, w))?;

	match block.last() {
		Some(v) => v.write(mng, w),
		None => Ok(()),
	}
}

impl Driver for Block {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let label = mng.push_label();

		match self.label_type() {
			Some(LabelType::Forward) => {
				write_inner_block(self, mng, w)?;
				write!(w, "::continue_at_{label}::")?;
			}
			Some(LabelType::Backward) => {
				write!(w, "::continue_at_{label}::")?;
				write!(w, "while true do ")?;
				write_inner_block(self, mng, w)?;

				if self.last().is_none() {
					write!(w, "break ")?;
				}

				write!(w, "end ")?;
			}
			None => write_inner_block(self, mng, w)?,
		}

		mng.pop_label();

		Ok(())
	}
}

impl Driver for BrIf {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "if ")?;
		Condition(self.condition()).write(w)?;
		write!(w, "then ")?;
		self.target().write(mng, w)?;
		write!(w, "end ")
	}
}

impl Driver for If {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "if ")?;
		Condition(self.condition()).write(w)?;
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

impl DriverNoContext for Call {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write_call_store(self.result(), w)?;

		write!(w, "FUNC_LIST[{}](", self.function())?;
		self.param_list().write(w)?;
		write!(w, ")")
	}
}

impl DriverNoContext for CallIndirect {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write_call_store(self.result(), w)?;

		write!(w, "TABLE_LIST[{}].data[", self.table())?;
		self.index().write(w)?;
		write!(w, "](")?;
		self.param_list().write(w)?;
		write!(w, ")")
	}
}

impl DriverNoContext for SetTemporary {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} = ", self.var())?;
		self.value().write(w)
	}
}

impl DriverNoContext for SetLocal {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "loc_{} = ", self.var())?;
		self.value().write(w)
	}
}

impl DriverNoContext for SetGlobal {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value = ", self.var())?;
		self.value().write(w)
	}
}

impl DriverNoContext for StoreAt {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "store_{}(memory_at_0, ", self.store_type().as_name())?;
		self.pointer().write(w)?;

		if self.offset() != 0 {
			write!(w, "+ {}", self.offset())?;
		}

		write!(w, ", ")?;
		self.value().write(w)?;
		write!(w, ")")
	}
}

impl DriverNoContext for MemoryGrow {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		let result = self.result();
		let memory = self.memory();

		write!(w, "reg_{result} = rt.allocator.grow(memory_at_{memory}, ")?;
		self.size().write(w)?;
		write!(w, ")")
	}
}

impl Driver for Statement {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Block(s) => s.write(mng, w),
			Self::BrIf(s) => s.write(mng, w),
			Self::If(s) => s.write(mng, w),
			Self::Call(s) => s.write(w),
			Self::CallIndirect(s) => s.write(w),
			Self::SetTemporary(s) => s.write(w),
			Self::SetLocal(s) => s.write(w),
			Self::SetGlobal(s) => s.write(w),
			Self::StoreAt(s) => s.write(w),
			Self::MemoryGrow(s) => s.write(w),
		}
	}
}

fn write_parameter_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	write!(w, "function(")?;
	write_ascending("loc", 0..ast.num_param(), w)?;
	write!(w, ")")
}

fn write_variable_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	let mut total = ast.num_param();

	for data in ast.local_data().iter().filter(|v| v.0 != 0) {
		let range = total..total + usize::try_from(data.0).unwrap();
		let typed = if data.1 == ValType::I64 { "0LL" } else { "0" }.as_bytes();

		total = range.end;

		write!(w, "local ")?;
		write_ascending("loc", range.clone(), w)?;
		write!(w, " = ")?;
		write_separated(range, |_, w| w.write_all(typed), w)?;
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
		let br_map = br_table::visit(self);

		write_parameter_list(self, w)?;
		write_variable_list(self, w)?;

		if !br_map.is_empty() {
			write!(w, "local br_map, temp = {{}}, nil ")?;
		}

		mng.set_table_map(br_map);
		self.code().write(mng, w)?;

		if self.num_result() != 0 {
			write!(w, "return ")?;
			write_ascending("reg", 0..self.num_result(), w)?;
			write!(w, " ")?;
		}

		write!(w, "end ")
	}
}
