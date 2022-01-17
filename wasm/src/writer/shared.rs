use std::{io::Result, ops::Range};

use parity_wasm::elements::{Internal, Module, NameSection, ResizableLimits};

use crate::ast::node::Function;

use super::base::Writer;

pub fn aux_internal_index(internal: Internal) -> u32 {
	match internal {
		Internal::Function(v) | Internal::Table(v) | Internal::Memory(v) | Internal::Global(v) => v,
	}
}

fn new_limit_max(limits: &ResizableLimits) -> String {
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

	write!(w, "rt.allocator.new({}, {})", a, b)
}

pub fn write_func_name(wasm: &Module, index: u32, offset: u32, w: Writer) -> Result<()> {
	let opt = wasm
		.names_section()
		.and_then(NameSection::functions)
		.and_then(|v| v.names().get(index));

	write!(w, "FUNC_LIST")?;

	if let Some(name) = opt {
		write!(w, "--[[{}]]", name)?;
	}

	write!(w, "[{}] =", index + offset)
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

pub fn write_parameter_list(func: &Function, w: Writer) -> Result<()> {
	write!(w, "function(")?;
	write_in_order("param", func.num_param, w)?;
	write!(w, ")")
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
	if !func.local_list.is_empty() {
		let num_local = func.local_list.len().try_into().unwrap();

		write!(w, "local ")?;
		write_in_order("loc", num_local, w)?;
		write!(w, " = ")?;

		for (i, t) in func.local_list.iter().enumerate() {
			if i != 0 {
				write!(w, ", ")?;
			}

			write!(w, "ZERO_{} ", t)?;
		}
	}

	if func.num_stack != 0 {
		write!(w, "local ")?;
		write_in_order("reg", func.num_stack, w)?;
		write!(w, " ")?;
	}

	Ok(())
}
