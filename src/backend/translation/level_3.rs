use super::level_2::{gen_function, gen_init_expression};
use crate::{
	backend::helper::{edition::Edition, writer::Writer},
	data::Module,
};
use parity_wasm::elements::{External, ImportCountType, Internal, ResizableLimits};
use std::io::Result;

const RUNTIME_DATA: &str = "
local grow_page_num = runtime.grow_page_num

local add = runtime.add
local sub = runtime.sub
local mul = runtime.mul
local div = runtime.div

local le = runtime.le
local lt = runtime.lt
local ge = runtime.ge
local gt = runtime.gt

local band = runtime.band
local bor = runtime.bor
local bxor = runtime.bxor
local bnot = runtime.bnot

local shl = runtime.shl
local shr = runtime.shr

local extend = runtime.extend
local wrap = runtime.wrap

local load = runtime.load
local store = runtime.store
";

fn gen_import_of<T>(m: &Module, w: Writer, lower: &str, cond: T) -> Result<()>
where
	T: Fn(&External) -> bool,
{
	let import = match m.parent.import_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let upper = lower.to_uppercase();

	for (i, v) in import.iter().filter(|v| cond(v.external())).enumerate() {
		let field = v.field();
		let module = v.module();

		writeln!(w, "{}[{}] = wasm.{}.{}.{}", upper, i, module, lower, field)?;
	}

	Ok(())
}

fn aux_internal_index(internal: Internal) -> u32 {
	match internal {
		Internal::Function(v) | Internal::Table(v) | Internal::Memory(v) | Internal::Global(v) => v,
	}
}

fn gen_export_of<T>(m: &Module, w: Writer, lower: &str, cond: T) -> Result<()>
where
	T: Fn(&Internal) -> bool,
{
	let export = match m.parent.export_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let upper = lower.to_uppercase();

	writeln!(w, "{} = {{", lower)?;

	for v in export.iter().filter(|v| cond(v.internal())) {
		let field = v.field();
		let index = aux_internal_index(*v.internal());

		writeln!(w, "{} = {}[{}],", field, upper, index)?;
	}

	writeln!(w, "}},")
}

fn gen_import_list(m: &Module, w: Writer) -> Result<()> {
	gen_import_of(m, w, "func_list", |v| matches!(v, External::Function(_)))?;
	gen_import_of(m, w, "table_list", |v| matches!(v, External::Table(_)))?;
	gen_import_of(m, w, "memory_list", |v| matches!(v, External::Memory(_)))?;
	gen_import_of(m, w, "global_list", |v| matches!(v, External::Global(_)))
}

fn gen_export_list(m: &Module, w: Writer) -> Result<()> {
	gen_export_of(m, w, "func_list", |v| matches!(v, Internal::Function(_)))?;
	gen_export_of(m, w, "table_list", |v| matches!(v, Internal::Table(_)))?;
	gen_export_of(m, w, "memory_list", |v| matches!(v, Internal::Memory(_)))?;
	gen_export_of(m, w, "global_list", |v| matches!(v, Internal::Global(_)))
}

fn gen_limit_data(limit: &ResizableLimits, w: Writer) -> Result<()> {
	writeln!(w, "{{ min = {}", limit.initial())?;

	if let Some(max) = limit.maximum() {
		writeln!(w, ", max = {}", max)?;
	}

	writeln!(w, ", data = {{}} }}")
}

fn gen_table_list(m: &Module, w: Writer) -> Result<()> {
	let table = match m.parent.table_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let offset = m.parent.import_count(ImportCountType::Table);

	for (i, v) in table.iter().enumerate() {
		let index = i + offset;

		writeln!(w, "TABLE_LIST[{}] =", index)?;
		gen_limit_data(v.limits(), w)?;
	}

	Ok(())
}

fn gen_memory_list(m: &Module, w: Writer) -> Result<()> {
	let memory = match m.parent.memory_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};
	let offset = m.parent.import_count(ImportCountType::Memory);

	for (i, v) in memory.iter().enumerate() {
		let index = i + offset;

		writeln!(w, "MEMORY_LIST[{}] =", index)?;
		gen_limit_data(v.limits(), w)?;
	}

	Ok(())
}

fn gen_global_list(m: &Module, w: Writer) -> Result<()> {
	let global = match m.parent.global_section() {
		Some(v) => v,
		None => return Ok(()),
	};
	let offset = m.parent.import_count(ImportCountType::Global);

	for (i, v) in global.entries().iter().enumerate() {
		let index = i + offset;

		writeln!(w, "GLOBAL_LIST[{}] = {{ value =", index)?;
		gen_init_expression(v.init_expr().code(), w)?;
		writeln!(w, "}}")?;
	}

	Ok(())
}

fn gen_element_list(m: &Module, w: Writer) -> Result<()> {
	let element = match m.parent.elements_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};

	for v in element {
		writeln!(w, "do")?;
		writeln!(w, "local target = TABLE_LIST[{}].data", v.index())?;
		writeln!(w, "local offset =")?;

		gen_init_expression(v.offset().as_ref().unwrap().code(), w)?;

		for (i, f) in v.members().iter().enumerate() {
			writeln!(w, "target[offset + {}] = FUNC_LIST[{}]", i, f)?;
		}

		writeln!(w, "end")?;
	}

	Ok(())
}

fn gen_data_list(m: &Module, w: Writer) -> Result<()> {
	let data = match m.parent.data_section() {
		Some(v) => v.entries(),
		None => return Ok(()),
	};

	for v in data {
		writeln!(w, "do")?;
		writeln!(w, "local target = MEMORY_LIST[{}].data", v.index())?;
		writeln!(w, "local offset =")?;

		gen_init_expression(v.offset().as_ref().unwrap().code(), w)?;

		writeln!(w, "/ 4")?;

		for (i, b) in v.value().chunks(4).enumerate() {
			let mut temp = [0; 4];

			temp.iter_mut().zip(b).for_each(|(l, r)| *l = *r);

			let value = u32::from_le_bytes(temp);

			writeln!(w, "target[offset + {}] = 0x{:X}", i, value)?;
		}

		writeln!(w, "end")?;
	}

	Ok(())
}

fn gen_start_point(m: &Module, w: Writer) -> Result<()> {
	writeln!(w, "local function run_init_code()")?;
	gen_table_list(m, w)?;
	gen_memory_list(m, w)?;
	gen_global_list(m, w)?;
	gen_element_list(m, w)?;
	gen_data_list(m, w)?;
	writeln!(w, "end")?;

	writeln!(w, "return function(wasm)")?;
	gen_import_list(m, w)?;
	writeln!(w, "run_init_code()")?;

	if let Some(start) = m.parent.start_section() {
		writeln!(w, "FUNC_LIST[{}]()", start)?;
	}

	writeln!(w, "return {{")?;
	gen_export_list(m, w)?;

	writeln!(w, "}} end")
}

fn gen_nil_array(name: &str, len: usize, w: Writer) -> Result<()> {
	if len == 0 {
		return Ok(());
	}

	let list = vec!["nil"; len].join(", ");

	writeln!(w, "local {} = {{[0] = {}}}", name, list)
}

pub fn translate(spec: &dyn Edition, m: &Module, w: Writer) -> Result<()> {
	writeln!(w, "local runtime = require({})", spec.runtime())?;
	writeln!(w, "{}", RUNTIME_DATA)?;

	gen_nil_array("FUNC_LIST", m.in_arity.len(), w)?;
	gen_nil_array("TABLE_LIST", m.parent.table_space(), w)?;
	gen_nil_array("MEMORY_LIST", m.parent.memory_space(), w)?;
	gen_nil_array("GLOBAL_LIST", m.parent.globals_space(), w)?;

	let offset = m.ex_arity.len();

	for i in 0..m.in_arity.len() {
		writeln!(w, "FUNC_LIST[{}] =", i + offset)?;

		gen_function(spec, i, m, w)?;
	}

	gen_start_point(m, w)
}
