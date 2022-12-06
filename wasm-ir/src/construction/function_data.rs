use std::collections::HashMap;

use wasmparser::{FunctionBody, Operator, OperatorsReader, Result, ValType};

use crate::module::{read_checked, TypeInfo};

use super::passes::{
	block_bound_tracking::{BlockBoundTracking, Bound},
	dead_code_elimination::DeadCodeElimination,
	max_stack_tracking::MaxStackTracking,
};

fn read_operators(reader: OperatorsReader) -> Result<Vec<Operator>> {
	let mut code = read_checked(reader)?;

	DeadCodeElimination::default().run(&mut code);

	Ok(code)
}

#[derive(Default)]
pub struct FunctionData<'c> {
	pub(crate) local_list: Vec<(u32, ValType)>,
	pub(crate) code: Vec<Operator<'c>>,

	pub(crate) bound_map: HashMap<usize, Bound>,
	pub(crate) max_stack_size: usize,

	pub(crate) param_count: usize,
	pub(crate) result_count: usize,
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
		let local_list = read_checked(body.get_locals_reader()?)?;
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

	pub fn bound(&self, index: usize) -> Bound {
		self.bound_map.get(&index).copied().unwrap()
	}
}
