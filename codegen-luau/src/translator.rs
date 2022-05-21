use std::{
	collections::BTreeSet,
	io::{Result, Write},
};

use parity_wasm::elements::{
	External, ImportCountType, Instruction, Internal, Module, NameSection, ResizableLimits,
};

use wasm_ast::{
	builder::{Builder, TypeInfo},
	node::Intermediate,
};

use crate::{
	analyzer::{localize, memory},
	backend::manager::{Driver, Manager},
};

fn to_internal_index(internal: Internal) -> u32 {
	match internal {
		Internal::Function(v) | Internal::Table(v) | Internal::Memory(v) | Internal::Global(v) => v,
	}
}

fn limit_data_of(limits: &ResizableLimits) -> (u32, u32) {
	let max = limits.maximum().unwrap_or(0xFFFF);

	(limits.initial(), max)
}

fn write_table_init(limit: &ResizableLimits, w: &mut dyn Write) -> Result<()> {
	let (a, b) = limit_data_of(limit);

	write!(w, "{{ min = {a}, max = {b}, data = {{}} }}")
}

fn write_memory_init(limit: &ResizableLimits, w: &mut dyn Write) -> Result<()> {
	let (a, b) = limit_data_of(limit);

	write!(w, "rt.allocator.new({a}, {b})")
}

fn write_named_array(name: &str, len: usize, w: &mut dyn Write) -> Result<()> {
	let len = len.saturating_sub(1);

	write!(w, "local {name} = table.create({len})")
}

fn write_constant(code: &[Instruction], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let func = Builder::new(type_info).with_anon(code);

	write!(w, "(")?;
	func.write(&mut Manager::default(), w)?;
	write!(w, ")()")
}

fn write_import_of<T>(wasm: &Module, lower: &str, cond: T, w: &mut dyn Write) -> Result<()>
where
	T: Fn(&External) -> bool,
{
	let import = match wasm.import_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let upper = lower.to_uppercase();

	for (i, v) in import.iter().filter(|v| cond(v.external())).enumerate() {
		let field = v.field();
		let module = v.module();

		write!(w, r#"{upper}[{i}] = wasm["{module}"].{lower}["{field}"]"#)?;
	}

	Ok(())
}

fn write_export_of<T>(wasm: &Module, lower: &str, cond: T, w: &mut dyn Write) -> Result<()>
where
	T: Fn(&Internal) -> bool,
{
	let export = match wasm.export_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let upper = lower.to_uppercase();

	write!(w, "{lower} = {{")?;

	for v in export.iter().filter(|v| cond(v.internal())) {
		let field = v.field();
		let index = to_internal_index(*v.internal());

		write!(w, r#"["{field}"] = {upper}[{index}],"#)?;
	}

	write!(w, "}},")
}

fn write_import_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	write_import_of(wasm, "func_list", |v| matches!(v, External::Function(_)), w)?;
	write_import_of(wasm, "table_list", |v| matches!(v, External::Table(_)), w)?;
	write_import_of(wasm, "memory_list", |v| matches!(v, External::Memory(_)), w)?;
	write_import_of(wasm, "global_list", |v| matches!(v, External::Global(_)), w)
}

fn write_export_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	write_export_of(wasm, "func_list", |v| matches!(v, Internal::Function(_)), w)?;
	write_export_of(wasm, "table_list", |v| matches!(v, Internal::Table(_)), w)?;
	write_export_of(wasm, "memory_list", |v| matches!(v, Internal::Memory(_)), w)?;
	write_export_of(wasm, "global_list", |v| matches!(v, Internal::Global(_)), w)
}

fn write_table_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	let table = match wasm.table_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let offset = wasm.import_count(ImportCountType::Table);

	for (i, v) in table.iter().enumerate() {
		write!(w, "TABLE_LIST[{}] =", i + offset)?;
		write_table_init(v.limits(), w)?;
	}

	Ok(())
}

fn write_memory_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	let memory = match wasm.memory_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let offset = wasm.import_count(ImportCountType::Memory);

	for (i, v) in memory.iter().enumerate() {
		write!(w, "MEMORY_LIST[{}] =", i + offset)?;
		write_memory_init(v.limits(), w)?;
	}

	Ok(())
}

fn write_global_list(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let global = match wasm.global_section() {
		Some(v) => v,
		None => return Ok(()),
	};
	let offset = wasm.import_count(ImportCountType::Global);

	for (i, v) in global.entries().iter().enumerate() {
		write!(w, "GLOBAL_LIST[{}] = {{ value =", i + offset)?;
		write_constant(v.init_expr().code(), type_info, w)?;
		write!(w, "}}")?;
	}

	Ok(())
}

fn write_element_list(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let element = match wasm.elements_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};

	for v in element {
		let code = v.offset().as_ref().unwrap().code();

		write!(w, "do ")?;
		write!(w, "local target = TABLE_LIST[{}].data ", v.index())?;
		write!(w, "local offset =")?;

		write_constant(code, type_info, w)?;

		write!(w, "local data = {{")?;

		v.members()
			.iter()
			.try_for_each(|v| write!(w, "FUNC_LIST[{v}],"))?;

		write!(w, "}}")?;

		write!(w, "table.move(data, 1, #data, offset, target)")?;

		write!(w, "end ")?;
	}

	Ok(())
}

fn write_data_list(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let data = match wasm.data_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};

	for v in data {
		let code = v.offset().as_ref().unwrap().code();

		write!(w, "do ")?;
		write!(w, "local target = MEMORY_LIST[{}]", v.index())?;
		write!(w, "local offset =")?;

		write_constant(code, type_info, w)?;

		write!(w, "local data = \"")?;

		v.value().iter().try_for_each(|v| write!(w, "\\x{v:02X}"))?;

		write!(w, "\"")?;

		write!(w, "rt.allocator.init(target, offset, data)")?;

		write!(w, "end ")?;
	}

	Ok(())
}

fn build_func_list(wasm: &Module, type_info: &TypeInfo) -> Vec<Intermediate> {
	let list = match wasm.code_section() {
		Some(v) => v.bodies(),
		None => return Vec::new(),
	};

	let iter = list.iter().enumerate();

	iter.map(|f| Builder::new(type_info).with_index(f.0, f.1))
		.collect()
}

fn write_local_operation(head: &str, tail: &str, w: &mut dyn Write) -> Result<()> {
	match (head, tail) {
		("abs" | "ceil" | "floor" | "sqrt" | "min" | "max", _)
		| ("band" | "bor" | "bxor", "i32") => {
			write!(w, "local {head}_{tail} = math.{head} ")
		}
		_ => write!(w, "local {head}_{tail} = rt.{head}.{tail} "),
	}
}

fn write_localize_used(func_list: &[Intermediate], w: &mut dyn Write) -> Result<()> {
	let loc_set: BTreeSet<_> = func_list.iter().flat_map(localize::visit).collect();

	loc_set
		.into_iter()
		.try_for_each(|(a, b)| write_local_operation(a, b, w))
}

fn write_memory_used(func_list: &[Intermediate], w: &mut dyn Write) -> Result<Vec<usize>> {
	let mem_set: BTreeSet<_> = func_list.iter().flat_map(memory::visit).collect();
	let list: Vec<_> = mem_set.into_iter().collect();

	for mem in &list {
		write!(w, "local memory_at_{mem} ")?;
	}

	Ok(list)
}

fn write_func_start(wasm: &Module, index: u32, offset: u32, w: &mut dyn Write) -> Result<()> {
	let opt = wasm
		.names_section()
		.and_then(NameSection::functions)
		.and_then(|v| v.names().get(index));

	write!(w, "FUNC_LIST")?;

	if let Some(name) = opt {
		write!(w, "--[[{name}]]")?;
	}

	write!(w, "[{}] =", index + offset)
}

fn write_func_list(
	wasm: &Module,
	type_info: &TypeInfo,
	func_list: &[Intermediate],
	w: &mut dyn Write,
) -> Result<()> {
	let offset = type_info.len_ex().try_into().unwrap();

	func_list.iter().enumerate().try_for_each(|(i, v)| {
		write_func_start(wasm, i.try_into().unwrap(), offset, w)?;

		v.write(&mut Manager::default(), w)
	})
}

fn write_module_start(
	wasm: &Module,
	type_info: &TypeInfo,
	mem_list: &[usize],
	w: &mut dyn Write,
) -> Result<()> {
	write!(w, "local function run_init_code()")?;
	write_table_list(wasm, w)?;
	write_memory_list(wasm, w)?;
	write_global_list(wasm, type_info, w)?;
	write_element_list(wasm, type_info, w)?;
	write_data_list(wasm, type_info, w)?;
	write!(w, "end ")?;

	write!(w, "return function(wasm)")?;
	write_import_list(wasm, w)?;
	write!(w, "run_init_code()")?;

	for mem in mem_list {
		write!(w, "memory_at_{mem} = MEMORY_LIST[{mem}]")?;
	}

	if let Some(start) = wasm.start_section() {
		write!(w, "FUNC_LIST[{start}]()")?;
	}

	write!(w, "return {{")?;
	write_export_list(wasm, w)?;
	write!(w, "}} end ")
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn translate(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let func_list = build_func_list(wasm, type_info);

	write_localize_used(&func_list, w)?;

	let mem_list = write_memory_used(&func_list, w)?;

	write_named_array("FUNC_LIST", wasm.functions_space(), w)?;
	write_named_array("TABLE_LIST", wasm.table_space(), w)?;
	write_named_array("MEMORY_LIST", wasm.memory_space(), w)?;
	write_named_array("GLOBAL_LIST", wasm.globals_space(), w)?;

	write_func_list(wasm, type_info, &func_list, w)?;
	write_module_start(wasm, type_info, &mem_list, w)
}
