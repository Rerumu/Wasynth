use std::collections::HashMap;

use wasmparser::{FunctionBody, LocalsReader, Operator, OperatorsReader, Result, ValType};

use crate::module::{read_checked, TypeInfo};

use super::passes::{
	block_bound_tracking::{BlockBoundTracking, Bound},
	dead_code_elimination::DeadCodeElimination,
	max_stack_tracking::MaxStackTracking,
};

fn read_locals(reader: LocalsReader) -> Result<Vec<(usize, ValType)>> {
	let map = |v: (u32, _)| (usize::try_from(v.0).unwrap(), v.1);

	reader.into_iter().map(|v| v.map(map)).collect()
}

fn read_operators(reader: OperatorsReader) -> Result<Vec<Operator>> {
	let mut code = read_checked(reader)?;

	DeadCodeElimination::default().run(&mut code);

	Ok(code)
}

#[derive(Default)]
pub struct FunctionData<'c> {
	local_list: Vec<(usize, ValType)>,
	code: Vec<Operator<'c>>,

	bound_map: HashMap<usize, Bound>,
	max_stack_size: usize,

	param_count: usize,
	result_count: usize,
}

impl<'c> FunctionData<'c> {
	/// # Errors
	///
	/// Returns an error if the function is malformed.
	pub fn from_function(
		body: &FunctionBody<'c>,
		index: usize,
		type_info: &TypeInfo,
	) -> Result<Self> {
		let local_list = read_locals(body.get_locals_reader()?)?;
		let code = read_operators(body.get_operators_reader()?)?;
		let ty = type_info.by_func_index(index);
		let bound_map = BlockBoundTracking::default().run(&code);
		let max_stack_size = MaxStackTracking::from(type_info).run(&code, ty.returns.len());

		Ok(Self {
			local_list,
			code,
			bound_map,
			max_stack_size,
			param_count: ty.params.len(),
			result_count: ty.returns.len(),
		})
	}

	/// # Errors
	///
	/// Returns an error if the function is malformed.
	pub fn from_expression(reader: OperatorsReader<'c>, type_info: &TypeInfo) -> Result<Self> {
		let code = read_operators(reader)?;
		let bound_map = BlockBoundTracking::default().run(&code);
		let max_stack_size = MaxStackTracking::from(type_info).run(&code, 1);

		Ok(Self {
			local_list: Vec::new(),
			code,
			bound_map,
			max_stack_size,
			param_count: 0,
			result_count: 1,
		})
	}

	pub fn local_list(&self) -> &[(usize, ValType)] {
		&self.local_list
	}

	pub fn local_count(&self) -> usize {
		self.local_list.iter().map(|v| v.0).sum()
	}

	pub fn code(&self) -> &[Operator<'c>] {
		&self.code
	}

	pub fn bound_map(&self) -> &HashMap<usize, Bound> {
		&self.bound_map
	}

	pub fn max_stack_size(&self) -> usize {
		self.max_stack_size
	}

	pub fn param_count(&self) -> usize {
		self.param_count
	}

	pub fn result_count(&self) -> usize {
		self.result_count
	}
}
