use std::{
	collections::BTreeSet,
	io::{Result, Write},
};

use wasm_ast::{
	factory::Factory,
	module::{External, Module, TypeInfo},
	node::{FuncData, Statement},
};
use wasmparser::{
	Data, DataKind, Element, ElementItem, ElementKind, Export, Import, InitExpr, Operator,
	OperatorsReader,
};

use crate::{
	analyzer::localize,
	backend::manager::{Driver, Manager},
};

trait AsIEName {
	fn as_ie_name(&self) -> &str;
}

impl AsIEName for External {
	fn as_ie_name(&self) -> &str {
		match self {
			External::Func => "func_list",
			External::Table => "table_list",
			External::Memory => "memory_list",
			External::Global => "global_list",
			External::Tag => unimplemented!(),
		}
	}
}

fn reader_to_code(reader: OperatorsReader) -> Vec<Operator> {
	let parsed: std::result::Result<_, _> = reader.into_iter().collect();

	parsed.unwrap()
}

fn write_named_array(name: &str, len: usize, w: &mut dyn Write) -> Result<()> {
	let len = match len.checked_sub(1) {
		Some(len) => len,
		None => return Ok(()),
	};

	write!(w, "local {name} = table_new({len}, 1)")
}

fn write_constant(init: &InitExpr, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let code = reader_to_code(init.get_operators_reader());
	let func = Factory::from_type_info(type_info).create_anonymous(&code);

	if let Some(Statement::SetTemporary(stat)) = func.code().code().last() {
		stat.value().write(&mut Manager::default(), w)
	} else {
		write!(w, r#"error("Valueless constant")"#)
	}
}

fn write_import_of(list: &[Import], wanted: External, w: &mut dyn Write) -> Result<()> {
	let lower = wanted.as_ie_name();
	let upper = lower.to_uppercase();

	for (i, Import { name, module, .. }) in list
		.iter()
		.filter(|v| External::from(v.ty) == wanted)
		.enumerate()
	{
		write!(w, r#"{upper}[{i}] = wasm["{module}"].{lower}["{name}"]"#)?;
	}

	Ok(())
}

fn write_export_of(list: &[Export], wanted: External, w: &mut dyn Write) -> Result<()> {
	let lower = wanted.as_ie_name();
	let upper = lower.to_uppercase();

	write!(w, "{lower} = {{")?;

	for Export { name, index, .. } in list.iter().filter(|v| External::from(v.kind) == wanted) {
		write!(w, r#"["{name}"] = {upper}[{index}],"#)?;
	}

	write!(w, "}},")
}

fn write_import_list(list: &[Import], w: &mut dyn Write) -> Result<()> {
	write_import_of(list, External::Func, w)?;
	write_import_of(list, External::Table, w)?;
	write_import_of(list, External::Memory, w)?;
	write_import_of(list, External::Global, w)
}

fn write_export_list(list: &[Export], w: &mut dyn Write) -> Result<()> {
	write_export_of(list, External::Func, w)?;
	write_export_of(list, External::Table, w)?;
	write_export_of(list, External::Memory, w)?;
	write_export_of(list, External::Global, w)
}

fn write_table_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Table);
	let table = wasm.table_section();

	for (i, table) in table.iter().enumerate() {
		let index = offset + i;
		let min = table.initial;
		let max = table.maximum.unwrap_or(0xFFFF);

		write!(
			w,
			"TABLE_LIST[{index}] = {{ min = {min}, max = {max}, data = {{}} }}"
		)?;
	}

	Ok(())
}

fn write_memory_list(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Memory);
	let memory = wasm.memory_section();

	for (i, ty) in memory.iter().enumerate() {
		let index = offset + i;
		let min = ty.initial;
		let max = ty.maximum.unwrap_or(0xFFFF);

		write!(w, "MEMORY_LIST[{index}] = rt.allocator.new({min}, {max})")?;
	}

	Ok(())
}

fn write_global_list(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Global);
	let global = wasm.global_section();

	for (i, global) in global.iter().enumerate() {
		let index = offset + i;

		write!(w, "GLOBAL_LIST[{index}] = {{ value =")?;
		write_constant(&global.init_expr, type_info, w)?;
		write!(w, "}}")?;
	}

	Ok(())
}

fn write_element_list(list: &[Element], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	for element in list {
		let (index, init) = match element.kind {
			ElementKind::Active {
				table_index,
				init_expr,
			} => (table_index, init_expr),
			_ => unimplemented!(),
		};

		write!(w, "do ")?;
		write!(w, "local target = TABLE_LIST[{index}].data ")?;
		write!(w, "local offset =")?;

		write_constant(&init, type_info, w)?;

		write!(w, "local data = {{")?;

		for item in element.items.get_items_reader().unwrap() {
			match item.unwrap() {
				ElementItem::Func(index) => write!(w, "FUNC_LIST[{index}],"),
				ElementItem::Expr(init) => write_constant(&init, type_info, w),
			}?;
		}

		write!(w, "}}")?;

		write!(w, "table.move(data, 1, #data, offset, target)")?;

		write!(w, "end ")?;
	}

	Ok(())
}

fn write_data_list(list: &[Data], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	for data in list {
		let (index, init) = match data.kind {
			DataKind::Passive => unimplemented!(),
			DataKind::Active {
				memory_index,
				init_expr,
			} => (memory_index, init_expr),
		};

		write!(w, "rt.store.string(")?;
		write!(w, "MEMORY_LIST[{index}],")?;
		write_constant(&init, type_info, w)?;
		write!(w, r#","{}")"#, data.data.escape_ascii())?;
	}

	Ok(())
}

fn build_func_list(wasm: &Module, type_info: &TypeInfo) -> Vec<FuncData> {
	let offset = wasm.import_count(External::Func);
	let mut builder = Factory::from_type_info(type_info);

	wasm.code_section()
		.iter()
		.enumerate()
		.map(|f| builder.create_indexed(f.0 + offset, f.1))
		.collect()
}

fn write_local_operation(head: &str, tail: &str, w: &mut dyn Write) -> Result<()> {
	write!(w, "local {head}_{tail} = ")?;

	match (head, tail) {
		("abs" | "ceil" | "floor" | "sqrt", _) => write!(w, "math.{head} "),
		("rem", "i32") => write!(w, "math.fmod "),
		("band" | "bor" | "bxor" | "bnot", _) => write!(w, "bit.{head} "),
		("shl", _) => write!(w, "bit.lshift "),
		("shr", "i32" | "i64") => write!(w, "bit.arshift "),
		("shr", "u32" | "u64") => write!(w, "bit.rshift "),
		("rotl", _) => write!(w, "bit.rol "),
		("rotr", _) => write!(w, "bit.ror "),
		("trunc", "u32_f32" | "u32_f64") => write!(w, "math.floor "),
		("convert", "f32_i64" | "f64_i64") => write!(w, "tonumber "),
		_ => write!(w, "rt.{head}.{tail} "),
	}
}

fn write_localize_used(func_list: &[FuncData], w: &mut dyn Write) -> Result<BTreeSet<usize>> {
	let mut loc_set = BTreeSet::new();
	let mut mem_set = BTreeSet::new();

	for (loc, mem) in func_list.iter().map(localize::visit) {
		loc_set.extend(loc);
		mem_set.extend(mem);
	}

	for loc in loc_set {
		write_local_operation(loc.0, loc.1, w)?;
	}

	for mem in &mem_set {
		write!(w, "local memory_at_{mem} ")?;
	}

	Ok(mem_set)
}

fn write_func_start(wasm: &Module, index: u32, w: &mut dyn Write) -> Result<()> {
	write!(w, "FUNC_LIST[{index}] =")?;

	match wasm.name_section().get(&index) {
		Some(name) => write!(w, "--[[ {name} ]]"),
		None => Ok(()),
	}
}

fn write_func_list(wasm: &Module, func_list: &[FuncData], w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Func);

	func_list.iter().enumerate().try_for_each(|(i, v)| {
		let index = (offset + i).try_into().unwrap();

		write_func_start(wasm, index, w)?;

		v.write(&mut Manager::default(), w)
	})
}

fn write_module_start(
	wasm: &Module,
	type_info: &TypeInfo,
	mem_set: &BTreeSet<usize>,
	w: &mut dyn Write,
) -> Result<()> {
	write!(w, "local function run_init_code()")?;
	write_table_list(wasm, w)?;
	write_memory_list(wasm, w)?;
	write_global_list(wasm, type_info, w)?;
	write_element_list(wasm.element_section(), type_info, w)?;
	write_data_list(wasm.data_section(), type_info, w)?;
	write!(w, "end ")?;

	write!(w, "return function(wasm)")?;
	write_import_list(wasm.import_section(), w)?;
	write!(w, "run_init_code()")?;

	for mem in mem_set {
		write!(w, "memory_at_{mem} = MEMORY_LIST[{mem}]")?;
	}

	if let Some(start) = wasm.start_section() {
		write!(w, "FUNC_LIST[{start}]()")?;
	}

	write!(w, "return {{")?;
	write_export_list(wasm.export_section(), w)?;
	write!(w, "}} end ")
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn from_inst_list(code: &[Operator], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	Factory::from_type_info(type_info)
		.create_anonymous(code)
		.write(&mut Manager::default(), w)
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn from_module_typed(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let func_list = build_func_list(wasm, type_info);
	let mem_set = write_localize_used(&func_list, w)?;

	write!(w, "local table_new = require(\"table.new\")")?;
	write_named_array("FUNC_LIST", wasm.function_space(), w)?;
	write_named_array("TABLE_LIST", wasm.table_space(), w)?;
	write_named_array("MEMORY_LIST", wasm.memory_space(), w)?;
	write_named_array("GLOBAL_LIST", wasm.global_space(), w)?;

	write_func_list(wasm, &func_list, w)?;
	write_module_start(wasm, type_info, &mem_set, w)
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn from_module_untyped(wasm: &Module, w: &mut dyn Write) -> Result<()> {
	let type_info = TypeInfo::from_module(wasm);

	from_module_typed(wasm, &type_info, w)
}
