use std::vec::Drain;

use generational_arena::Arena;
use wasmparser::{BlockType, FuncType, MemoryImmediate, Operator};

use crate::{
	graph::{
		discriminant::{
			BinOpType, CastOpType, CmpOpType, ICmpOpType, LoadType, StoreType, UnOpType,
		},
		node::{
			Argument, BinOp, Call, CastOp, CmpOp, Edge, EdgeList, Gamma, GetFunction,
			GetTableElement, GlobalGet, GlobalSet, Lambda, Load, MemoryGrow, MemorySize, Node,
			Number, Store, Theta, ToArena, UnOp, Undefined, Unreachable,
		},
	},
	module::TypeInfo,
};

use super::{function_data::FunctionData, passes::block_bound_tracking::Bound};

#[derive(Default)]
struct SpecialEdge {
	memory_edge: Edge,
	global_edge: Edge,
	case_edge: Edge,
	looping_edge: Edge,
}

impl SpecialEdge {
	fn get_all_orders(&self) -> (Edge, Edge) {
		(self.memory_edge, self.global_edge)
	}

	fn set_all_orders(&mut self, edge: Edge, start: usize) {
		self.memory_edge = edge.set_port(start);
		self.global_edge = edge.set_port(start + 1);
	}

	fn set_all(&mut self, edge: Edge, start: usize) {
		self.memory_edge = edge.set_port(start);
		self.global_edge = edge.set_port(start + 1);
		self.case_edge = edge.set_port(start + 2);
		self.looping_edge = edge.set_port(start + 3);
	}

	fn state_count() -> usize {
		2
	}

	fn out_count() -> usize {
		3
	}
}

// Data order across top level:
// [Locals..., Max..., Memory, Global, Case]
//
// NOTE 1: Locals includes arguments; first local is first argument.
// NOTE 2: Max is the maximum stack size calculated automatically.

#[derive(Default, Clone, Copy)]
struct PortMap {
	param_count: usize,
	local_count: usize,
	max_stack_size: usize,
}

impl PortMap {
	fn get_stack_start(self) -> usize {
		self.param_count + self.local_count
	}

	fn get_stack_end(self) -> usize {
		self.get_stack_start() + self.max_stack_size
	}

	fn get_case_port(self, edge: Edge) -> Edge {
		edge.set_port(self.get_stack_end() + 2)
	}

	fn get_looping_port(self, edge: Edge) -> Edge {
		edge.set_port(self.get_stack_end() + 3)
	}

	fn get_gamma_iter(self, edge: Edge) -> impl Iterator<Item = Edge> {
		edge.set_port_range(0..self.get_stack_end() + SpecialEdge::out_count())
	}

	fn get_param_iter(self, edge: Edge) -> impl Iterator<Item = Edge> {
		edge.set_port_range(0..self.param_count)
	}

	fn get_local_iter(self, edge: Edge) -> impl Iterator<Item = Edge> {
		edge.set_port_range(self.param_count..self.get_stack_start())
	}

	fn get_stack_iter(self, edge: Edge) -> impl Iterator<Item = Edge> {
		edge.set_port_range(self.get_stack_start()..self.get_stack_end())
	}
}

impl From<&FunctionData<'_>> for PortMap {
	fn from(function: &FunctionData) -> Self {
		Self {
			param_count: function.param_count(),
			local_count: function.local_count(),
			max_stack_size: function.max_stack_size(),
		}
	}
}

#[derive(Default)]
pub struct SimpleBuilder {
	arena: Arena<Node>,

	port_map: PortMap,

	special: SpecialEdge,

	last_block_id: usize,
	local_list: Vec<Edge>,
	value_stack: Vec<Edge>,
}

impl SimpleBuilder {
	fn set_to_function(&mut self, function: &FunctionData) {
		let port_map = PortMap::from(function);
		let local_count = port_map.param_count + port_map.local_count;

		self.port_map = port_map;

		self.local_list.clear();
		self.value_stack.clear();

		self.last_block_id = function.bound_map().len();

		self.local_list.reserve_exact(local_count);
		self.value_stack.reserve_exact(port_map.max_stack_size);
	}

	fn reset(&mut self, stack_count: usize) -> Edge {
		self.local_list.clear();
		self.value_stack.clear();

		let param_edge = Argument.to_arena(&mut self.arena);
		let port_map = self.port_map;

		self.local_list.extend(port_map.get_param_iter(param_edge));
		self.local_list.extend(port_map.get_local_iter(param_edge));

		self.value_stack
			.extend(port_map.get_stack_iter(param_edge).take(stack_count));

		self.special.set_all(param_edge, port_map.get_stack_end());

		param_edge
	}

	fn reset_in_place(&mut self) -> Edge {
		self.reset(self.value_stack.len())
	}

	fn load_repeated<T>(&mut self, len: usize, start: T, buffer: &mut Vec<Edge>)
	where
		T: ToArena + Clone,
	{
		buffer.extend(std::iter::repeat_with(|| start.clone().to_arena(&mut self.arena)).take(len));
	}

	fn load_theta_in(&mut self, function: &FunctionData, buffer: &mut Vec<Edge>) {
		for &(len, ty) in function.local_list() {
			self.load_repeated(len, Number::zero_of(ty), buffer);
		}

		self.load_repeated(function.max_stack_size(), Undefined, buffer);
	}

	fn drain_stack(&mut self, len: usize) -> Drain<Edge> {
		self.value_stack.drain(self.value_stack.len() - len..)
	}

	fn add_unreachable(&mut self, result_count: usize) {
		let (memory_order, global_order) = self.special.get_all_orders();
		let edge = Unreachable {
			memory_order,
			global_order,
		}
		.to_arena(&mut self.arena);

		let offset = SpecialEdge::state_count();

		self.value_stack.clear();
		self.value_stack
			.extend(edge.set_port_range(offset..result_count + offset));

		self.special.set_all_orders(edge, 0);
	}

	fn push_get_function(&mut self, function: usize) {
		let edge = GetFunction { function }.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_get_table_element(&mut self, table: usize) {
		let index = self.value_stack.pop().unwrap();
		let edge = GetTableElement { table, index }.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_call_with_type(&mut self, ty: &FuncType) {
		let (memory_order, global_order) = self.special.get_all_orders();
		let edge = Call {
			function: self.value_stack.pop().unwrap(),
			argument_list: self.drain_stack(ty.params.len()).collect(),
			memory_order,
			global_order,
		}
		.to_arena(&mut self.arena);

		let ret_iter = edge.set_port_range(0..ty.returns.len());

		self.value_stack.extend(ret_iter);
		self.special.set_all_orders(edge, ty.params.len());
	}

	fn push_select(&mut self) {
		let condition = self.value_stack.pop().unwrap();
		let list_in = vec![
			self.value_stack.pop().unwrap(),
			self.value_stack.pop().unwrap(),
		]
		.into();

		let edge = Gamma::select(condition, list_in, &mut self.arena).to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_get_local(&mut self, local_index: u32) {
		let local_index = usize::try_from(local_index).unwrap();
		let edge = self.local_list[local_index];

		self.value_stack.push(edge);
	}

	fn add_set_local(&mut self, local_index: u32) {
		let local_index = usize::try_from(local_index).unwrap();
		let edge = self.value_stack.pop().unwrap();

		self.local_list[local_index] = edge;
	}

	fn add_tee_local(&mut self, local_index: u32) {
		let local_index = usize::try_from(local_index).unwrap();
		let edge = *self.value_stack.last().unwrap();

		self.local_list[local_index] = edge;
	}

	fn push_get_global(&mut self, global_index: u32) {
		let edge = GlobalGet {
			global: global_index.try_into().unwrap(),
			order: self.special.global_edge,
		}
		.to_arena(&mut self.arena);

		self.special.global_edge = edge.set_port(1);
		self.value_stack.push(edge);
	}

	fn add_set_global(&mut self, global_index: u32) {
		let edge = GlobalSet {
			global: global_index.try_into().unwrap(),
			value: self.value_stack.pop().unwrap(),
			order: self.special.global_edge,
		}
		.to_arena(&mut self.arena);

		self.special.global_edge = edge;
	}

	fn push_load(&mut self, load_type: LoadType, memarg: MemoryImmediate) {
		let edge = Load {
			load_type,
			memory: memarg.memory.try_into().unwrap(),
			offset: memarg.offset,
			pointer: self.value_stack.pop().unwrap(),
			order: self.special.memory_edge,
		}
		.to_arena(&mut self.arena);

		self.special.memory_edge = edge.set_port(1);
		self.value_stack.push(edge);
	}

	fn add_store(&mut self, store_type: StoreType, memarg: MemoryImmediate) {
		let edge = Store {
			store_type,
			memory: memarg.memory.try_into().unwrap(),
			offset: memarg.offset,
			value: self.value_stack.pop().unwrap(),
			pointer: self.value_stack.pop().unwrap(),
			order: self.special.memory_edge,
		}
		.to_arena(&mut self.arena);

		self.special.memory_edge = edge;
	}

	fn push_memory_size(&mut self, memory: u32) {
		let edge = MemorySize {
			memory: memory.try_into().unwrap(),
			order: self.special.memory_edge,
		}
		.to_arena(&mut self.arena);

		self.special.memory_edge = edge.set_port(1);
		self.value_stack.push(edge);
	}

	fn push_memory_grow(&mut self, memory: u32) {
		let edge = MemoryGrow {
			memory: memory.try_into().unwrap(),
			delta: self.value_stack.pop().unwrap(),
			order: self.special.memory_edge,
		}
		.to_arena(&mut self.arena);

		self.special.memory_edge = edge.set_port(1);
		self.value_stack.push(edge);
	}

	fn push_constant<T: Into<Number>>(&mut self, value: T) {
		let edge = value.into().to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_un_op(&mut self, op_type: UnOpType) {
		let edge = UnOp {
			op_type,
			rhs: self.value_stack.pop().unwrap(),
		}
		.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_bin_op(&mut self, op_type: BinOpType) {
		let edge = BinOp {
			op_type,
			lhs: self.value_stack.pop().unwrap(),
			rhs: self.value_stack.pop().unwrap(),
		}
		.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_cmp_op(&mut self, op_type: CmpOpType) {
		let edge = CmpOp {
			op_type,
			lhs: self.value_stack.pop().unwrap(),
			rhs: self.value_stack.pop().unwrap(),
		}
		.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	fn push_cast_op(&mut self, op_type: CastOpType) {
		let edge = CastOp {
			op_type,
			rhs: self.value_stack.pop().unwrap(),
		}
		.to_arena(&mut self.arena);

		self.value_stack.push(edge);
	}

	// Eqz is the only unary comparison so it's "emulated"
	// using a constant operand
	fn try_add_equal_zero(&mut self, op: &Operator) -> bool {
		match op {
			Operator::I32Eqz => {
				self.push_constant(0_i32);
				self.push_cmp_op(CmpOpType::I32(ICmpOpType::Eq));

				true
			}
			Operator::I64Eqz => {
				self.push_constant(0_i64);
				self.push_cmp_op(CmpOpType::I64(ICmpOpType::Eq));

				true
			}
			_ => false,
		}
	}

	fn try_add_operation(&mut self, op: &Operator) -> bool {
		if let Ok((load_type, memarg)) = LoadType::try_extract(op) {
			self.push_load(load_type, memarg);
		} else if let Ok((store_type, memarg)) = StoreType::try_extract(op) {
			self.add_store(store_type, memarg);
		} else if let Ok(op_type) = UnOpType::try_from(op) {
			self.push_un_op(op_type);
		} else if let Ok(op_type) = BinOpType::try_from(op) {
			self.push_bin_op(op_type);
		} else if let Ok(op_type) = CmpOpType::try_from(op) {
			self.push_cmp_op(op_type);
		} else if let Ok(op_type) = CastOpType::try_from(op) {
			self.push_cast_op(op_type);
		} else if !self.try_add_equal_zero(op) {
			return false;
		}

		true
	}

	// The locals, stack, and special edges are loaded into the buffer.
	fn load_gamma_stack(&self, param_edge: Edge) -> Vec<Edge> {
		let mut list = Vec::new();
		let stack_iter = self.port_map.get_stack_iter(param_edge);

		list.extend(&self.local_list);
		list.extend(stack_iter);
		list.push(self.special.memory_edge);
		list.push(self.special.global_edge);

		list
	}

	// The condition and destination for the next state are loaded into the buffer.
	fn load_gamma_branch(&mut self, destination: usize, buffer: &mut Vec<Edge>) {
		let condition = (destination != self.last_block_id).into();
		let destination = destination.try_into().unwrap();

		buffer.push(Number::I32(destination).to_arena(&mut self.arena));
		buffer.push(Number::I32(condition).to_arena(&mut self.arena));
	}

	// The return operation is prepared by loading the stack,
	// then the branch, and finally values are moved to the
	// base of the target.
	fn load_return(&mut self, param_edge: Edge, base: usize, target: Target) -> EdgeList {
		let mut list = self.load_gamma_stack(param_edge);

		self.load_gamma_branch(target.id, &mut list);

		let result_start = self.port_map.get_stack_start() + base;

		for (i, edge) in self.drain_stack(target.result_count).enumerate() {
			list[result_start + i] = edge;
		}

		list.into()
	}
}

#[derive(Clone, Copy)]
struct Target {
	id: usize,
	result_count: usize,
}

struct BlockState {
	fallthrough: Target,
	destination: Target,
	base: usize,
	start: usize,
}

pub struct LambdaBuilder<'t> {
	type_info: &'t TypeInfo<'t>,
	simple: SimpleBuilder,

	param_edge: Edge,

	gamma_list: Vec<(Edge, EdgeList)>,
	block_stack: Vec<BlockState>,
}

impl<'t> LambdaBuilder<'t> {
	fn push_call(&mut self, function_index: u32) {
		let function_index = function_index.try_into().unwrap();
		let ty = self.type_info.by_func_index(function_index);

		self.simple.push_get_function(function_index);
		self.simple.push_call_with_type(ty);
	}

	fn push_call_indirect(&mut self, type_index: u32, table_index: u32) {
		let type_index = type_index.try_into().unwrap();
		let table_index = table_index.try_into().unwrap();
		let ty = self.type_info.by_type_index(type_index);

		self.simple.push_get_table_element(table_index);
		self.simple.push_call_with_type(ty);
	}

	fn start_state(&mut self, start: usize, fallthrough: Target, destination: Target) {
		let base = self.simple.value_stack.len();

		self.block_stack.push(BlockState {
			fallthrough,
			destination,
			base,
			start,
		});
	}

	fn start_machine(&mut self, result_count: usize) {
		let destination = Target {
			id: self.simple.last_block_id,
			result_count,
		};

		self.start_state(0, destination, destination);
		self.param_edge = self.simple.reset(0);
	}

	fn reset_if_not_stacked(&mut self, start: usize, param_count: usize) {
		let last = self.block_stack.last().unwrap();

		if last.start != start {
			self.param_edge = self.simple.reset(param_count);
		}
	}

	fn start_block(&mut self, ty: BlockType, bound: Bound) {
		let (param_count, result_count) = self.type_info.by_block_type(ty);
		let destination = Target {
			id: bound.end,
			result_count,
		};

		self.reset_if_not_stacked(bound.start, param_count);
		self.start_state(bound.start, destination, destination);
	}

	fn start_loop(&mut self, ty: BlockType, bound: Bound) {
		let (param_count, result_count) = self.type_info.by_block_type(ty);
		let destination = Target {
			id: bound.start,
			result_count: param_count,
		};

		let fallthrough = Target {
			id: bound.end,
			result_count,
		};

		self.reset_if_not_stacked(bound.start, param_count);
		self.start_state(bound.start, fallthrough, destination);
	}

	fn add_unreachable(&mut self) {
		let last = self.block_stack.last().unwrap();
		let result_count = last.fallthrough.result_count;

		self.simple.add_unreachable(result_count);
		self.add_br_unconditional(0);
	}

	fn load_return_relative(&mut self, param_edge: Edge, relative_depth: u32) -> EdgeList {
		let index = self.block_stack.len() - 1 - usize::try_from(relative_depth).unwrap();
		let state = &self.block_stack[index];

		self.simple
			.load_return(param_edge, state.base, state.destination)
	}

	fn add_br_unconditional(&mut self, relative_depth: u32) {
		let param_edge = self.param_edge;
		let list_out = self.load_return_relative(param_edge, relative_depth);
		let last = self.block_stack.pop().unwrap();

		// If there is a block we are exiting to then we create
		// a new stack frame.
		if !self.block_stack.is_empty() {
			self.param_edge = self.simple.reset(last.base + last.fallthrough.result_count);
		}

		self.gamma_list.push((param_edge, list_out));
	}

	fn get_target_for_fallthrough(&self) -> Target {
		let id = self.gamma_list.len();
		let result_count = self.simple.value_stack.len();

		Target { id, result_count }
	}

	fn add_br_if(&mut self, relative_depth: u32) {
		let condition = self.simple.value_stack.pop().unwrap();

		let param_in = self.param_edge;
		let list_in = self.simple.load_gamma_stack(param_in).into();

		let fallthrough = self.get_target_for_fallthrough();
		let param_1 = self.simple.reset_in_place();
		let list_1 = self.simple.load_return(param_1, 0, fallthrough);

		let param_2 = self.simple.reset_in_place();
		let list_2 = self.load_return_relative(param_2, relative_depth);

		self.param_edge = self.simple.reset_in_place();

		let gamma = Gamma {
			condition,
			list_in,
			list_out: vec![(param_1, list_1), (param_2, list_2)],
		}
		.to_arena(&mut self.simple.arena);

		let list_out = self.simple.port_map.get_gamma_iter(gamma).collect();

		self.gamma_list.push((param_in, list_out));
	}

	fn end_block(&mut self) {
		let last = self.block_stack.last_mut().unwrap();

		last.destination = last.fallthrough;

		self.add_br_unconditional(0);
	}

	fn build_block_list(&mut self, function: &FunctionData) -> Vec<(Edge, EdgeList)> {
		let mut iter = function.code().iter().enumerate();

		self.start_machine(function.result_count());

		while let Some((i, inst)) = iter.next() {
			if self.simple.try_add_operation(inst) {
				continue;
			}

			match *inst {
				Operator::Unreachable => {
					self.add_unreachable();

					// Instructions after unconditional branches leave the
					// stack in an undefined state.
					iter.next().unwrap();
				}
				Operator::Nop => {}
				Operator::Block { ty } => self.start_block(ty, function.bound_map()[&i]),
				Operator::Loop { ty } => self.start_loop(ty, function.bound_map()[&i]),
				Operator::End => self.end_block(),
				Operator::Br { relative_depth } => {
					self.add_br_unconditional(relative_depth);

					iter.next().unwrap();
				}
				Operator::BrIf { relative_depth } => self.add_br_if(relative_depth),
				Operator::Return => {
					let relative_depth = u32::try_from(self.block_stack.len() - 1).unwrap();

					self.add_br_unconditional(relative_depth);

					iter.next().unwrap();
				}
				Operator::Call { function_index } => self.push_call(function_index),
				Operator::CallIndirect {
					index, table_index, ..
				} => self.push_call_indirect(index, table_index),
				Operator::Drop => {
					self.simple.value_stack.pop().unwrap();
				}
				Operator::Select => self.simple.push_select(),
				Operator::LocalGet { local_index } => self.simple.push_get_local(local_index),
				Operator::LocalSet { local_index } => self.simple.add_set_local(local_index),
				Operator::LocalTee { local_index } => self.simple.add_tee_local(local_index),
				Operator::GlobalGet { global_index } => self.simple.push_get_global(global_index),
				Operator::GlobalSet { global_index } => self.simple.add_set_global(global_index),
				Operator::MemorySize { mem, .. } => self.simple.push_memory_size(mem),
				Operator::MemoryGrow { mem, .. } => self.simple.push_memory_grow(mem),
				Operator::I32Const { value } => self.simple.push_constant(value),
				Operator::I64Const { value } => self.simple.push_constant(value),
				Operator::F32Const { value } => self.simple.push_constant(value.bits()),
				Operator::F64Const { value } => self.simple.push_constant(value.bits()),
				_ => unimplemented!("{inst:?}"),
			}
		}

		std::mem::take(&mut self.gamma_list)
	}

	fn build_gamma(&mut self, theta_edge: Edge, function: &FunctionData) -> Gamma {
		let condition = self.simple.port_map.get_case_port(theta_edge);

		let list_in = self.simple.port_map.get_gamma_iter(theta_edge).collect();
		let list_out = self.build_block_list(function);

		Gamma {
			condition,
			list_in,
			list_out,
		}
	}

	fn build_theta(&mut self, func_edge: Edge, function: &FunctionData) -> Theta {
		let param_edge = Argument.to_arena(&mut self.simple.arena);
		let gamma = self
			.build_gamma(param_edge, function)
			.to_arena(&mut self.simple.arena);

		let mut list_in = EdgeList::default();
		let data = &mut list_in.data;
		let port_map = self.simple.port_map;

		// Add parameters from the function first.
		data.extend(port_map.get_param_iter(func_edge));

		// Then add locals, which are zero-initialized.
		// Also add stack space, which is undefined.
		self.simple.load_theta_in(function, data);

		// Then add the state edges, which are "locals".
		data.extend(func_edge.set_port_range(data.len()..data.len() + SpecialEdge::state_count()));

		// Lastly, initialize the state machine to the first case.
		data.push(Number::I32(0).to_arena(&mut self.simple.arena));

		let list_out = port_map.get_gamma_iter(gamma).collect();
		let condition = port_map.get_looping_port(gamma);

		Theta {
			param_edge,
			list_in,
			list_out,
			condition,
		}
	}

	pub fn build(&mut self, function: &FunctionData) -> Lambda {
		let param_edge = Argument.to_arena(&mut self.simple.arena);

		self.simple.set_to_function(function);

		let theta = self
			.build_theta(param_edge, function)
			.to_arena(&mut self.simple.arena);

		let list_out = self
			.simple
			.port_map
			.get_stack_iter(theta)
			.take(function.result_count())
			.collect();

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
			param_edge: Edge::default(),
			gamma_list: Vec::new(),
			block_stack: Vec::new(),
		}
	}
}
