use std::{
	collections::BTreeSet,
	io::{Result, Write},
};

use parity_wasm::elements::{
	External, ImportCountType, Instruction, Internal, Module as WasmModule, ResizableLimits,
};

use crate::backend::{
	ast::{data::Function, transformer::Transformer},
	edition::data::Edition,
	visitor::localize,
};

use super::{arity::List as ArityList, writer::Data};

fn aux_internal_index(internal: Internal) -> u32 {
	match internal {
		Internal::Function(v) | Internal::Table(v) | Internal::Memory(v) | Internal::Global(v) => v,
	}
}

fn gen_table_init(limit: &ResizableLimits, w: &mut dyn Write) -> Result<()> {
	write!(w, "{{ min = {}", limit.initial())?;

	if let Some(max) = limit.maximum() {
		write!(w, ", max = {}", max)?;
	}

	write!(w, ", data = {{}} }}")
}

fn gen_memory_init(limit: &ResizableLimits, w: &mut dyn Write) -> Result<()> {
	write!(w, "rt.memory.new({}, ", limit.initial())?;

	if let Some(max) = limit.maximum() {
		write!(w, "{}", max)?;
	} else {
		write!(w, "nil")?;
	}

	write!(w, ")")
}

fn gen_nil_array(name: &str, len: usize, w: &mut dyn Write) -> Result<()> {
	if len == 0 {
		return Ok(());
	}

	write!(w, "local {} = {{[0] = {}}}", name, "nil, ".repeat(len))
}

pub fn gen_expression(code: &[Instruction], w: &mut dyn Write) -> Result<()> {
	assert!(code.len() == 2);

	let inst = code.first().unwrap();

	match *inst {
		Instruction::I32Const(v) => write!(w, "{} ", v),
		Instruction::I64Const(v) => write!(w, "{} ", v),
		Instruction::F32Const(v) => write!(w, "{} ", f32::from_bits(v)),
		Instruction::F64Const(v) => write!(w, "{} ", f64::from_bits(v)),
		Instruction::GetGlobal(i) => write!(w, "GLOBAL_LIST[{}].value ", i),
		_ => unreachable!(),
	}
}

pub struct Module<'a> {
	wasm: &'a WasmModule,
	arity: ArityList,
}

impl<'a> Module<'a> {
	pub fn new(wasm: &'a WasmModule) -> Self {
		let arity = ArityList::new(wasm);

		Self { wasm, arity }
	}

	fn gen_import_of<T>(&self, w: &mut dyn Write, lower: &str, cond: T) -> Result<()>
	where
		T: Fn(&External) -> bool,
	{
		let import = match self.wasm.import_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};
		let upper = lower.to_uppercase();

		for (i, v) in import.iter().filter(|v| cond(v.external())).enumerate() {
			let field = v.field();
			let module = v.module();

			write!(w, "{}[{}] = wasm.{}.{}.{} ", upper, i, module, lower, field)?;
		}

		Ok(())
	}

	fn gen_export_of<T>(&self, w: &mut dyn Write, lower: &str, cond: T) -> Result<()>
	where
		T: Fn(&Internal) -> bool,
	{
		let export = match self.wasm.export_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};
		let upper = lower.to_uppercase();

		write!(w, "{} = {{", lower)?;

		for v in export.iter().filter(|v| cond(v.internal())) {
			let field = v.field();
			let index = aux_internal_index(*v.internal());

			write!(w, "{} = {}[{}],", field, upper, index)?;
		}

		write!(w, "}},")
	}

	fn gen_import_list(&self, w: &mut dyn Write) -> Result<()> {
		self.gen_import_of(w, "func_list", |v| matches!(v, External::Function(_)))?;
		self.gen_import_of(w, "table_list", |v| matches!(v, External::Table(_)))?;
		self.gen_import_of(w, "memory_list", |v| matches!(v, External::Memory(_)))?;
		self.gen_import_of(w, "global_list", |v| matches!(v, External::Global(_)))
	}

	fn gen_export_list(&self, w: &mut dyn Write) -> Result<()> {
		self.gen_export_of(w, "func_list", |v| matches!(v, Internal::Function(_)))?;
		self.gen_export_of(w, "table_list", |v| matches!(v, Internal::Table(_)))?;
		self.gen_export_of(w, "memory_list", |v| matches!(v, Internal::Memory(_)))?;
		self.gen_export_of(w, "global_list", |v| matches!(v, Internal::Global(_)))
	}

	fn gen_table_list(&self, w: &mut dyn Write) -> Result<()> {
		let table = match self.wasm.table_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};
		let offset = self.wasm.import_count(ImportCountType::Table);

		for (i, v) in table.iter().enumerate() {
			let index = i + offset;

			write!(w, "TABLE_LIST[{}] =", index)?;
			gen_table_init(v.limits(), w)?;
		}

		Ok(())
	}

	fn gen_memory_list(&self, w: &mut dyn Write) -> Result<()> {
		let memory = match self.wasm.memory_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};
		let offset = self.wasm.import_count(ImportCountType::Memory);

		for (i, v) in memory.iter().enumerate() {
			let index = i + offset;

			write!(w, "MEMORY_LIST[{}] =", index)?;
			gen_memory_init(v.limits(), w)?;
		}

		Ok(())
	}

	fn gen_global_list(&self, w: &mut dyn Write) -> Result<()> {
		let global = match self.wasm.global_section() {
			Some(v) => v,
			None => return Ok(()),
		};
		let offset = self.wasm.import_count(ImportCountType::Global);

		for (i, v) in global.entries().iter().enumerate() {
			let index = i + offset;

			write!(w, "GLOBAL_LIST[{}] = {{ value =", index)?;

			gen_expression(v.init_expr().code(), w)?;

			write!(w, "}}")?;
		}

		Ok(())
	}

	fn gen_element_list(&self, w: &mut dyn Write) -> Result<()> {
		let element = match self.wasm.elements_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};

		for v in element {
			write!(w, "do ")?;
			write!(w, "local target = TABLE_LIST[{}].data ", v.index())?;
			write!(w, "local offset =")?;

			gen_expression(v.offset().as_ref().unwrap().code(), w)?;

			write!(w, "local data = {{")?;

			v.members()
				.iter()
				.try_for_each(|v| write!(w, "FUNC_LIST[{}],", v))?;

			write!(w, "}}")?;

			write!(w, "table.move(data, 1, #data, offset, target)")?;

			write!(w, "end ")?;
		}

		Ok(())
	}

	fn gen_data_list(&self, w: &mut dyn Write) -> Result<()> {
		let data = match self.wasm.data_section() {
			Some(v) => v.entries(),
			None => return Ok(()),
		};

		for v in data {
			write!(w, "do ")?;
			write!(w, "local target = MEMORY_LIST[{}]", v.index())?;
			write!(w, "local offset =")?;

			gen_expression(v.offset().as_ref().unwrap().code(), w)?;

			write!(w, "local data = \"")?;

			v.value()
				.iter()
				.try_for_each(|v| write!(w, "\\x{:02X}", v))?;

			write!(w, "\"")?;

			write!(w, "rt.memory.init(target, offset, data)")?;

			write!(w, "end ")?;
		}

		Ok(())
	}

	fn gen_start_point(&self, w: &mut dyn Write) -> Result<()> {
		write!(w, "local function run_init_code()")?;
		self.gen_table_list(w)?;
		self.gen_memory_list(w)?;
		self.gen_global_list(w)?;
		self.gen_element_list(w)?;
		self.gen_data_list(w)?;
		write!(w, "end ")?;

		write!(w, "return function(wasm)")?;
		self.gen_import_list(w)?;
		write!(w, "run_init_code()")?;

		if let Some(start) = self.wasm.start_section() {
			write!(w, "FUNC_LIST[{}]()", start)?;
		}

		write!(w, "return {{")?;
		self.gen_export_list(w)?;
		write!(w, "}} end ")
	}

	fn gen_localize(&self, func_list: &[Function], w: &mut dyn Write) -> Result<()> {
		let mut loc_set = BTreeSet::new();

		for func in func_list {
			loc_set.extend(localize::visit(func));
		}

		loc_set
			.into_iter()
			.try_for_each(|(a, b)| write!(w, "local {0}_{1} = rt.{0}.{1} ", a, b))
	}

	pub fn translate(&self, ed: &dyn Edition, w: &mut dyn Write) -> Result<()> {
		write!(w, "local rt = require({})", ed.runtime())?;

		let func_list: Vec<_> = (0..self.arity.in_arity.len())
			.map(|i| Transformer::new(self.wasm, &self.arity, i).consume())
			.collect();

		self.gen_localize(&func_list, w)?;

		gen_nil_array("FUNC_LIST", self.wasm.functions_space(), w)?;
		gen_nil_array("TABLE_LIST", self.wasm.table_space(), w)?;
		gen_nil_array("MEMORY_LIST", self.wasm.memory_space(), w)?;
		gen_nil_array("GLOBAL_LIST", self.wasm.globals_space(), w)?;

		let offset = self.arity.ex_arity.len();

		for (i, v) in func_list.into_iter().enumerate() {
			write!(w, "FUNC_LIST[{}] =", i + offset)?;

			v.output(&mut Data::new(v.num_param, ed), w)?;
		}

		self.gen_start_point(w)
	}
}
