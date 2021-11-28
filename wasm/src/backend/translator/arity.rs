use std::convert::TryInto;

use parity_wasm::elements::{External, FunctionType, ImportEntry, Module, Type};

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

pub struct List {
	pub ex_arity: Vec<Arity>,
	pub in_arity: Vec<Arity>,
}

impl List {
	pub fn new(parent: &Module) -> Self {
		Self {
			ex_arity: Self::new_arity_ex_list(parent),
			in_arity: Self::new_arity_in_list(parent),
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

	fn new_arity_in_list(wasm: &Module) -> Vec<Arity> {
		let (types, funcs) = match (wasm.type_section(), wasm.function_section()) {
			(Some(t), Some(f)) => (t.types(), f.entries()),
			_ => return Vec::new(),
		};

		funcs
			.iter()
			.map(|i| Arity::from_index(types, i.type_ref()))
			.collect()
	}

	fn new_arity_ex_list(wasm: &Module) -> Vec<Arity> {
		let (types, imports) = match (wasm.type_section(), wasm.import_section()) {
			(Some(t), Some(i)) => (t.types(), i.entries()),
			_ => return Vec::new(),
		};

		imports
			.iter()
			.filter_map(|i| Self::new_arity_ext(types, i))
			.collect()
	}
}
