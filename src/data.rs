use crate::backend::helper::writer::ordered_iter;
use parity_wasm::elements::{
	External, FunctionType, ImportEntry, Instruction, Local, Module as WasmModule, Type,
};
use std::{borrow::Cow, convert::TryInto};

pub struct Code<'a> {
	pub num_local: u32,
	pub inst_list: &'a [Instruction],
	var_list: Vec<String>,
}

impl<'a> Code<'a> {
	pub fn new(inst_list: &'a [Instruction], num_local: u32) -> Self {
		Self {
			num_local,
			inst_list,
			var_list: Vec::new(),
		}
	}

	pub fn local_sum(list: &[Local]) -> u32 {
		list.iter().map(Local::count).sum()
	}

	pub fn var_name_of(&self, index: u32) -> Cow<'_, str> {
		let index: usize = index.try_into().unwrap();
		let offset = self.var_list.len();

		self.var_list
			.get(index)
			.map_or_else(|| format!("reg_{}", index - offset + 1).into(), Cow::from)
	}

	pub fn var_range_of(&self, start: u32, len: u32) -> Vec<Cow<'_, str>> {
		(start..start + len).map(|i| self.var_name_of(i)).collect()
	}
}

pub struct Arity {
	pub num_param: u32,
	pub num_result: u32,
}

impl Arity {
	fn from_type(typ: &FunctionType) -> Self {
		let num_param = typ.params().len().try_into().unwrap();
		let num_result = typ.results().len().try_into().unwrap();

		Self {
			num_param,
			num_result,
		}
	}

	pub fn from_index(types: &[Type], index: u32) -> Self {
		let Type::Function(typ) = &types[index as usize];

		Self::from_type(typ)
	}
}

pub struct Module<'a> {
	pub ex_arity: Vec<Arity>,
	pub in_arity: Vec<Arity>,
	pub code: Vec<Code<'a>>,
	pub parent: &'a WasmModule,
}

impl<'a> Module<'a> {
	pub fn new(parent: &'a WasmModule) -> Self {
		let mut module = Module {
			in_arity: Self::new_arity_in_list(parent),
			ex_arity: Self::new_arity_ex_list(parent),
			code: Self::new_function_list(parent),
			parent,
		};

		module.fill_cache();
		module
	}

	fn fill_cache(&mut self) {
		for (a, c) in self.in_arity.iter().zip(self.code.iter_mut()) {
			c.var_list = ordered_iter("param", a.num_param)
				.chain(ordered_iter("var", c.num_local))
				.collect();
		}
	}

	pub fn arity_of(&self, index: usize) -> &Arity {
		let offset = self.ex_arity.len();

		self.ex_arity
			.get(index)
			.or_else(|| self.in_arity.get(index - offset))
			.unwrap()
	}

	fn new_arity_ext(types: &[Type], import: &ImportEntry) -> Option<Arity> {
		if let External::Function(i) = import.external() {
			Some(Arity::from_index(types, *i))
		} else {
			None
		}
	}

	fn new_arity_in_list(wasm: &WasmModule) -> Vec<Arity> {
		let (types, funcs) = match (wasm.type_section(), wasm.function_section()) {
			(Some(t), Some(f)) => (t.types(), f.entries()),
			_ => return Vec::new(),
		};

		funcs
			.iter()
			.map(|i| Arity::from_index(types, i.type_ref()))
			.collect()
	}

	fn new_arity_ex_list(wasm: &WasmModule) -> Vec<Arity> {
		let (types, imports) = match (wasm.type_section(), wasm.import_section()) {
			(Some(t), Some(i)) => (t.types(), i.entries()),
			_ => return Vec::new(),
		};

		imports
			.iter()
			.filter_map(|i| Self::new_arity_ext(types, i))
			.collect()
	}

	fn new_function_list(wasm: &WasmModule) -> Vec<Code> {
		let bodies = match wasm.code_section() {
			Some(b) => b.bodies(),
			None => return Vec::new(),
		};

		bodies
			.iter()
			.map(|v| {
				let num_local = Code::local_sum(v.locals());

				Code::new(v.code().elements(), num_local)
			})
			.collect()
	}
}
