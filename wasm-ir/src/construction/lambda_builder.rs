use generational_arena::Arena;
use wasmparser::ValType;

use crate::{
	graph::node::{Argument, Edge, Gamma, Lambda, Node, Number, Theta, ToArena, Undefined},
	module::TypeInfo,
};

use super::function_data::FunctionData;

// In-edge order of data:
// [Locals..., Max..., Memory, Global, Case, Looping]
//
// NOTE 1: Locals includes arguments; first local is first argument.
// NOTE 2: Max is the maximum stack size calculated automatically.

#[derive(Default)]
struct SpecialEdge {
	memory_edge: Edge,
	global_edge: Edge,
	case_edge: Edge,
	looping_edge: Edge,
}

impl SpecialEdge {
	fn set_all(&mut self, edge: Edge, start: usize) {
		self.memory_edge = edge.set_port(start);
		self.global_edge = edge.set_port(start + 1);
		self.case_edge = edge.set_port(start + 2);
		self.looping_edge = edge.set_port(start + 3);
	}

	fn state_count() -> usize {
		2
	}

	fn count() -> usize {
		4
	}
}

#[derive(Default)]
pub struct SimpleBuilder {
	arena: Arena<Node>,

	param_edge: Edge,

	special: SpecialEdge,

	local_list: Vec<Edge>,
	value_stack: Vec<Edge>,
}

impl SimpleBuilder {
	fn reset(&mut self, function: &FunctionData) {
		self.local_list.clear();
		self.value_stack.clear();

		let locals = 0..function.param_count + function.local_list.len();
		let params = locals.end..locals.end + function.max_stack_size;

		self.param_edge = Argument.to_arena(&mut self.arena);

		self.special.set_all(self.param_edge, params.end);

		self.local_list
			.extend(self.param_edge.set_port_range(locals));

		self.value_stack
			.extend(self.param_edge.set_port_range(params));
	}

	fn load_repeated<T>(&mut self, len: usize, start: T, buffer: &mut Vec<Edge>)
	where
		T: ToArena + Clone,
	{
		buffer.extend(std::iter::repeat_with(|| start.clone().to_arena(&mut self.arena)).take(len));
	}

	fn load_local_list(&mut self, local_list: &[(u32, ValType)], buffer: &mut Vec<Edge>) {
		for &(len, ty) in local_list {
			let len = len.try_into().unwrap();

			self.load_repeated(len, Number::zero_of(ty), buffer);
		}
	}

	fn load_stack_list(&mut self, max_stack_size: usize, buffer: &mut Vec<Edge>) {
		self.load_repeated(max_stack_size, Undefined, buffer);
	}
}

pub struct LambdaBuilder<'t> {
	type_info: &'t TypeInfo<'t>,
	simple: SimpleBuilder,
}

impl<'t> LambdaBuilder<'t> {
	fn build_gamma(&mut self, theta_edge: Edge, function: &FunctionData) -> Gamma {
		let param_max = function.local_list.len() + function.param_count + function.max_stack_size;

		let condition = theta_edge.set_port(param_max + SpecialEdge::state_count());
		let list_in = theta_edge
			.set_port_range(0..param_max + SpecialEdge::count())
			.collect();

		Gamma {
			condition,
			list_in,
			list_out: Vec::new(),
		}
	}

	fn build_theta(&mut self, func_edge: Edge, function: &FunctionData) -> Theta {
		let param_edge = Argument.to_arena(&mut self.simple.arena);
		let gamma = self
			.build_gamma(param_edge, function)
			.to_arena(&mut self.simple.arena);

		let mut data = Vec::new();

		// Add parameters from the function first.
		data.extend(func_edge.set_port_range(0..function.param_count));

		// Then add the locals, which are zero-initialized.
		self.simple.load_local_list(&function.local_list, &mut data);

		// Then add the stack space, which is undefined.
		self.simple
			.load_stack_list(function.max_stack_size, &mut data);

		// Then add the state edges, which are "parameters".
		data.extend(func_edge.set_port_range(
			function.param_count..function.param_count + SpecialEdge::state_count(),
		));

		// Lastly, initialize the state machine to the first case.
		data.push(Number::I32(0).to_arena(&mut self.simple.arena));
		data.push(Undefined.to_arena(&mut self.simple.arena));

		let last_edge = data.len() - 1;

		Theta {
			param_edge,
			list_in: data.into(),
			list_out: gamma.set_port_range(0..last_edge).collect(),
			condition: gamma.set_port(last_edge),
		}
	}

	pub fn build(&mut self, function: &FunctionData) -> Lambda {
		let param_edge = Argument.to_arena(&mut self.simple.arena);
		let theta = self
			.build_theta(param_edge, function)
			.to_arena(&mut self.simple.arena);

		let stack_start = function.param_count + function.local_list.len();
		let result_range =
			stack_start..stack_start + function.result_count + SpecialEdge::state_count();

		let list_out = theta.set_port_range(result_range).collect();
		let arena = std::mem::take(&mut self.simple.arena);

		Lambda {
			arena,
			param_edge,
			list_out,
		}
	}
}

impl<'t> From<&'t TypeInfo<'t>> for LambdaBuilder<'t> {
	fn from(type_info: &'t TypeInfo<'t>) -> Self {
		Self {
			type_info,
			simple: SimpleBuilder::default(),
		}
	}
}
