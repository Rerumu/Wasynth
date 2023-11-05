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
	ConstExpr, Data, DataKind, Element, ElementItems, ElementKind, Export, Import, Operator,
	OperatorsReader, ValType,
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
			Self::Func => "func_list",
			Self::Table => "table_list",
			Self::Memory => "memory_list",
			Self::Global => "global_list",
			Self::Tag => unimplemented!(),
		}
	}
}

fn reader_to_code(reader: OperatorsReader) -> Vec<Operator> {
	let parsed: std::result::Result<_, _> = reader.into_iter().collect();

	parsed.unwrap()
}

fn write_named_array(name: &str, len: usize, w: &mut dyn Write) -> Result<()> {
	let Some(len) = len.checked_sub(1) else {
		return Ok(());
	};

	writeln!(w, "local {name} = table.create({len})")
}

fn write_constant(init: &ConstExpr, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let code = reader_to_code(init.get_operators_reader());
	let func = Factory::from_type_info(type_info).create_anonymous(&code);

	if let Some(Statement::SetTemporary(stat)) = func.code().code().last() {
		stat.value().write(&mut Manager::empty(), w)
	} else {
		writeln!(w, r#"error("Valueless constant")"#)
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
		write!(w, "\t")?;
		writeln!(w, r#"{upper}[{i}] = wasm["{module}"].{lower}["{name}"]"#)?;
	}

	Ok(())
}

fn write_export_of(list: &[Export], wanted: External, w: &mut dyn Write) -> Result<()> {
	let lower = wanted.as_ie_name();
	let upper = lower.to_uppercase();

	writeln!(w, "\t\t{lower} = {{")?;

	for Export { name, index, .. } in list.iter().filter(|v| External::from(v.kind) == wanted) {
		write!(w, "\t\t\t")?;
		writeln!(w, r#"["{name}"] = {upper}[{index}],"#)?;
	}

	writeln!(w, "\t\t}},")
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
		let min = table.ty.initial;
		let max = table.ty.maximum.unwrap_or(0xFFFF);

		writeln!(
			w,
			"\tTABLE_LIST[{index}] = {{ min = {min}, max = {max}, data = {{}} }}"
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

		writeln!(w, "\tMEMORY_LIST[{index}] = rt.allocator.new({min}, {max})")?;
	}

	Ok(())
}

fn write_global_list(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Global);
	let global = wasm.global_section();

	for (i, global) in global.iter().enumerate() {
		let index = offset + i;

		write!(w, "\tGLOBAL_LIST[{index}] = {{ value = ")?;
		write_constant(&global.init_expr, type_info, w)?;
		writeln!(w, " }}")?;
	}

	Ok(())
}

fn write_element_list(list: &[Element], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	for element in list {
		let ElementKind::Active {
			table_index: index,
			offset_expr: init,
		} = element.kind
		else {
			unimplemented!("passive elements not supported")
		};

		let index = index.unwrap_or(0);

		writeln!(w, "\tdo")?;
		writeln!(w, "\t\tlocal target = TABLE_LIST[{index}].data")?;
		write!(w, "\t\tlocal offset = ")?;

		write_constant(&init, type_info, w)?;

		writeln!(w)?;
		write!(w, "\t\tlocal data = {{ ")?;

		match element.items.clone() {
			ElementItems::Functions(functions) => {
				for index in functions {
					let index = index.unwrap();
					write!(w, "FUNC_LIST[{index}],")?;
				}
			}
			ElementItems::Expressions(expressions) => {
				for init in expressions {
					let init = init.unwrap();
					write_constant(&init, type_info, w)?;
				}
			}
		}

		writeln!(w, " }}")?;
		writeln!(w, "\t\ttable.move(data, 1, #data, offset, target)")?;
		writeln!(w, "\tend")?;
	}

	Ok(())
}

fn write_data_list(list: &[Data], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	for data in list {
		let (index, init) = match data.kind {
			DataKind::Passive => unimplemented!("passive data not supported"),
			DataKind::Active {
				memory_index,
				offset_expr,
			} => (memory_index, offset_expr),
		};

		write!(w, "\trt.store.string(MEMORY_LIST[{index}], ")?;
		write_constant(&init, type_info, w)?;
		writeln!(w, r#","{}")"#, data.data.escape_ascii())?;
	}

	Ok(())
}

fn build_func_list(wasm: &Module, type_info: &TypeInfo) -> Vec<FuncData> {
	let offset = wasm.import_count(External::Func);
	let mut builder = Factory::from_type_info(type_info);

	wasm.code_section()
		.iter()
		.enumerate()
		.map(|f| builder.create_indexed(f.0 + offset, f.1).unwrap())
		.collect()
}

fn write_local_operation(head: &str, tail: &str, w: &mut dyn Write) -> Result<()> {
	write!(w, "local {head}_{tail} = ")?;

	match (head, tail) {
		("abs" | "ceil" | "floor" | "sqrt", _) => write!(w, "math.{head}"),
		("band" | "bor" | "bxor" | "bnot", "i32") => write!(w, "bit32.{head}"),
		("clz", "i32") => write!(w, "bit32.countlz"),
		("ctz", "i32") => write!(w, "bit32.countrz"),
		_ => write!(w, "rt.{head}.{tail}"),
	}?;

	writeln!(w)
}

fn write_localize_used(
	wasm: &Module,
	func_list: &[FuncData],
	w: &mut dyn Write,
) -> Result<BTreeSet<usize>> {
	let mut loc_set = BTreeSet::new();
	let mut mem_set = BTreeSet::new();

	let has_global_i64 = wasm
		.global_section()
		.iter()
		.any(|g| g.ty.content_type == ValType::I64);

	if has_global_i64 {
		loc_set.insert(("i64", "ZERO"));
		loc_set.insert(("i64", "ONE"));
		loc_set.insert(("i64", "from_u32"));
	}

	for (loc, mem) in func_list.iter().map(localize::visit) {
		loc_set.extend(loc);
		mem_set.extend(mem);
	}

	for loc in loc_set {
		write_local_operation(loc.0, loc.1, w)?;
	}

	for mem in &mem_set {
		writeln!(w, "local memory_at_{mem}")?;
	}

	Ok(mem_set)
}

fn write_func_start(wasm: &Module, index: u32, w: &mut dyn Write) -> Result<()> {
	write!(w, "FUNC_LIST[{index}] = ")?;

	wasm.name_section()
		.get(&index)
		.map_or_else(|| Ok(()), |name| write!(w, "--[[ {name} ]] "))
}

fn write_func_list(wasm: &Module, func_list: &[FuncData], w: &mut dyn Write) -> Result<()> {
	let offset = wasm.import_count(External::Func);

	func_list.iter().enumerate().try_for_each(|(i, v)| {
		let index = (offset + i).try_into().unwrap();

		write_func_start(wasm, index, w)?;

		v.write(&mut Manager::function(v), w)
	})
}

fn write_module_start(
	wasm: &Module,
	type_info: &TypeInfo,
	mem_set: &BTreeSet<usize>,
	w: &mut dyn Write,
) -> Result<()> {
	writeln!(w, "local function run_init_code()")?;
	write_table_list(wasm, w)?;
	write_memory_list(wasm, w)?;
	write_global_list(wasm, type_info, w)?;
	write_element_list(wasm.element_section(), type_info, w)?;
	write_data_list(wasm.data_section(), type_info, w)?;
	writeln!(w, "end")?;

	writeln!(w, "return function(wasm)")?;
	write_import_list(wasm.import_section(), w)?;
	writeln!(w, "\trun_init_code()")?;

	for mem in mem_set {
		writeln!(w, "\tmemory_at_{mem} = MEMORY_LIST[{mem}]")?;
	}

	if let Some(start) = wasm.start_section() {
		writeln!(w, "\tFUNC_LIST[{start}]()")?;
	}

	writeln!(w, "\treturn {{")?;
	write_export_list(wasm.export_section(), w)?;
	writeln!(w, "\t}}")?;
	writeln!(w, "end")
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn from_inst_list(code: &[Operator], type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let ast = Factory::from_type_info(type_info).create_anonymous(code);

	ast.write(&mut Manager::function(&ast), w)
}

/// # Errors
/// Returns `Err` if writing to `Write` failed.
pub fn from_module_typed(wasm: &Module, type_info: &TypeInfo, w: &mut dyn Write) -> Result<()> {
	let func_list = build_func_list(wasm, type_info);
	let mem_set = write_localize_used(wasm, &func_list, w)?;

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
