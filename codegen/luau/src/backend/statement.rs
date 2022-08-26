use std::{
	io::{Result, Write},
	ops::Range,
};

use wasm_ast::node::{
	Block, Br, BrIf, BrTable, Call, CallIndirect, FuncData, If, LabelType, MemoryGrow, MemoryCopy,MemoryFill, SetGlobal,
	SetLocal, SetTemporary, Statement, StoreAt, Terminator,
};
use wasmparser::ValType;

use crate::{analyzer::br_target, indentation, indented, line};

use super::{
	expression::Condition,
	manager::{write_ascending, write_separated, Driver, DriverNoContext, Manager},
};

impl Driver for Br {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if !self.align().is_aligned() {
			indentation!(mng, w)?;
			write_ascending("reg", self.align().new_range(), w)?;
			write!(w, " = ")?;
			write_ascending("reg", self.align().old_range(), w)?;
			writeln!(w)?;
		}

		if self.target() == 0 {
			if let Some(Some(LabelType::Backward)) = mng.label_list().last() {
				line!(mng, w, "continue")
			} else {
				line!(mng, w, "break")
			}
		} else {
			let level = mng.label_list().len() - 1 - self.target();

			line!(mng, w, "desired = {level}")?;
			line!(mng, w, "break")
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
		line!(mng, w, "if temp < {} then", br.target())?;
		mng.indent();
		write_search_layer(range.start..center, list, mng, w)?;
		mng.dedent();
		indented!(mng, w, "else")?;
	}

	if range.end != center + 1 {
		writeln!(w, "if temp > {} then", br.target())?;
		mng.indent();
		write_search_layer(center + 1..range.end, list, mng, w)?;
		mng.dedent();
		indented!(mng, w, "else")?;
	}

	writeln!(w)?;
	mng.indent();
	br.write(mng, w)?;
	mng.dedent();
	line!(mng, w, "end")
}

fn write_table_setup(table: &BrTable, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	let id = mng.get_table_index(table);

	line!(mng, w, "if not br_map[{id}] then")?;
	mng.indent();
	line!(mng, w, "br_map[{id}] = (function()")?;
	mng.indent();
	indented!(mng, w, "return {{ [0] = ")?;

	table
		.data()
		.iter()
		.try_for_each(|v| write!(w, "{}, ", v.target()))?;

	writeln!(w, "}}")?;
	mng.dedent();
	line!(mng, w, "end)()")?;
	mng.dedent();
	line!(mng, w, "end")?;

	indented!(mng, w, "temp = br_map[{id}][")?;
	table.condition().write(w)?;
	writeln!(w, "] or {}", table.default().target())
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
			Self::Unreachable => line!(mng, w, r#"error("out of code bounds")"#),
			Self::Br(s) => s.write(mng, w),
			Self::BrTable(s) => s.write(mng, w),
		}
	}
}

fn write_br_parent(mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	if mng.label_list().iter().all(Option::is_none) {
		return Ok(());
	}

	line!(mng, w, "if desired then")?;
	mng.indent();

	if let Some(last) = *mng.label_list().last().unwrap() {
		let level = mng.label_list().len() - 1;

		line!(mng, w, "if desired == {level} then")?;
		mng.indent();
		line!(mng, w, "desired = nil")?;

		if last == LabelType::Backward {
			line!(mng, w, "continue")?;
		}

		mng.dedent();
		line!(mng, w, "end")?;
	}

	line!(mng, w, "break")?;

	mng.dedent();
	line!(mng, w, "end")
}

impl Driver for Block {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		mng.push_label(self.label_type());

		line!(mng, w, "while true do")?;
		mng.indent();

		self.code().iter().try_for_each(|s| s.write(mng, w))?;

		match self.last() {
			Some(v) => v.write(mng, w)?,
			None => line!(mng, w, "break")?,
		}

		mng.dedent();
		line!(mng, w, "end")?;

		mng.pop_label();
		write_br_parent(mng, w)
	}
}

impl Driver for BrIf {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		indented!(mng, w, "if ")?;
		Condition(self.condition()).write(w)?;
		writeln!(w, " then")?;
		mng.indent();
		self.target().write(mng, w)?;
		mng.dedent();
		line!(mng, w, "end")
	}
}

impl Driver for If {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		indented!(mng, w, "if ")?;
		Condition(self.condition()).write(w)?;
		writeln!(w, " then")?;

		mng.indent();
		self.on_true().write(mng, w)?;
		mng.dedent();

		if let Some(v) = self.on_false() {
			line!(mng, w, "else")?;
			mng.indent();
			v.write(mng, w)?;
			mng.dedent();
		}

		line!(mng, w, "end")
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
			write!(w, " + {}", self.offset())?;
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

impl DriverNoContext for MemoryCopy {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		let dst = self.dst();
		let src = self.src();
		let size = self.size();

		write!(w, "store.copy(memory_at_0, {dst}, {src}, ")?;
		size.write(w)?;
		write!(w, ")")
	}
}

impl DriverNoContext for MemoryFill {
	fn write(&self, w: &mut dyn Write) -> Result<()> {
		let mem = self.mem();
		let value = self.value();
		let n = self.n();

		write!(w, "store.copy(memory_at_0, {mem}, ")?;
		value.write(w)?;
		write!(w, ", ")?;
		n.write(w)?;
		write!(w, ")")
	}
}

fn write_stat(stat: &dyn DriverNoContext, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	indentation!(mng, w)?;
	stat.write(w)?;
	writeln!(w)
}

impl Driver for Statement {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Block(s) => s.write(mng, w),
			Self::BrIf(s) => s.write(mng, w),
			Self::If(s) => s.write(mng, w),
			Self::Call(s) => write_stat(s, mng, w),
			Self::CallIndirect(s) => write_stat(s, mng, w),
			Self::SetTemporary(s) => write_stat(s, mng, w),
			Self::SetLocal(s) => write_stat(s, mng, w),
			Self::SetGlobal(s) => write_stat(s, mng, w),
			Self::StoreAt(s) => write_stat(s, mng, w),
			Self::MemoryGrow(s) => write_stat(s, mng, w),
			Self::MemoryCopy(s) => write_stat(s, mng, w),
			Self::MemoryFill(s) => write_stat(s, mng, w),
		}
	}
}

fn write_parameter_list(ast: &FuncData, w: &mut dyn Write) -> Result<()> {
	write!(w, "function(")?;
	write_ascending("loc", 0..ast.num_param(), w)?;
	writeln!(w, ")")
}

fn write_variable_list(ast: &FuncData, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
	let mut total = ast.num_param();

	for data in ast.local_data().iter().filter(|v| v.0 != 0) {
		let range = total..total + usize::try_from(data.0).unwrap();
		let zero = if data.1 == ValType::I64 {
			"i64_ZERO"
		} else {
			"0"
		};

		total = range.end;

		indented!(mng, w, "local ")?;
		write_ascending("loc", range.clone(), w)?;
		write!(w, " = ")?;
		write_separated(range, |_, w| w.write_all(zero.as_bytes()), w)?;
		writeln!(w)?;
	}

	if ast.num_stack() != 0 {
		indented!(mng, w, "local ")?;
		write_ascending("reg", 0..ast.num_stack(), w)?;
		writeln!(w)?;
	}

	Ok(())
}

impl Driver for FuncData {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let br_data = br_target::visit(self);

		mng.indent();

		write_parameter_list(self, w)?;
		write_variable_list(self, mng, w)?;

		if br_data.1 {
			line!(mng, w, "local desired")?;
		}

		if !br_data.0.is_empty() {
			line!(mng, w, "local br_map = {{}}")?;
		}

		mng.set_table_map(br_data.0);
		self.code().write(mng, w)?;

		if self.num_result() != 0 {
			indented!(mng, w, "return ")?;
			write_ascending("reg", 0..self.num_result(), w)?;
			writeln!(w)?;
		}

		mng.dedent();

		line!(mng, w, "end")
	}
}
