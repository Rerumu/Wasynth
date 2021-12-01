use std::{io::Result, ops::Range};

use parity_wasm::elements::{Internal, ResizableLimits};

use crate::ast::node::Function;

use super::visit::Writer;

pub fn aux_internal_index(internal: Internal) -> u32 {
	match internal {
		Internal::Function(v) | Internal::Table(v) | Internal::Memory(v) | Internal::Global(v) => v,
	}
}

pub fn new_limit_max(limits: &ResizableLimits) -> String {
	match limits.maximum() {
		Some(v) => v.to_string(),
		None => "0xFFFF".to_string(),
	}
}

pub fn write_table_init(limit: &ResizableLimits, w: Writer) -> Result<()> {
	let a = limit.initial();
	let b = new_limit_max(limit);

	write!(w, "{{ min = {}, max = {}, data = {{}} }}", a, b)
}

pub fn write_memory_init(limit: &ResizableLimits, w: Writer) -> Result<()> {
	let a = limit.initial();
	let b = new_limit_max(limit);

	write!(w, "rt.memory.new({}, {})", a, b)
}

pub fn write_nil_array(name: &str, len: usize, w: Writer) -> Result<()> {
	if len == 0 {
		return Ok(());
	}

	write!(w, "local {} = {{[0] = {}}}", name, "nil, ".repeat(len))
}

pub fn write_in_order(prefix: &str, len: u32, w: Writer) -> Result<()> {
	if len == 0 {
		return Ok(());
	}

	write!(w, "{}_{}", prefix, 0)?;
	(1..len).try_for_each(|i| write!(w, ", {}_{}", prefix, i))
}

pub fn write_f32(f: f32, w: Writer) -> Result<()> {
	let sign = if f.is_sign_negative() { "-" } else { "" };

	if f.is_infinite() {
		write!(w, "{}math.huge ", sign)
	} else if f.is_nan() {
		write!(w, "{}0/0 ", sign)
	} else {
		write!(w, "{:e} ", f)
	}
}

pub fn write_f64(f: f64, w: Writer) -> Result<()> {
	let sign = if f.is_sign_negative() { "-" } else { "" };

	if f.is_infinite() {
		write!(w, "{}math.huge ", sign)
	} else if f.is_nan() {
		write!(w, "{}0/0 ", sign)
	} else {
		write!(w, "{:e} ", f)
	}
}

pub fn write_result_list(range: Range<u32>, w: Writer) -> Result<()> {
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

pub fn write_variable_list(func: &Function, w: Writer) -> Result<()> {
	if func.num_local != 0 {
		let list = vec!["0"; func.num_local as usize].join(", ");

		write!(w, "local ")?;
		write_in_order("loc", func.num_local, w)?;
		write!(w, " = {} ", list)?;
	}

	if func.num_stack != 0 {
		write!(w, "local ")?;
		write_in_order("reg", func.num_stack, w)?;
		write!(w, " ")?;
	}

	Ok(())
}
