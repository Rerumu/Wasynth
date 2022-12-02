use std::ops::Range;

use wasmparser::ValType;

use generational_arena::{Arena, Index};

use super::discriminant::{BinOpType, CastOpType, CmpOpType, LoadType, StoreType, UnOpType};

#[derive(Clone, Copy)]
pub struct Edge {
	node: Index,
	port: usize,
}

impl Edge {
	pub fn set_port(self, port: usize) -> Self {
		Self {
			node: self.node,
			port,
		}
	}

	pub fn set_port_range(self, range: Range<usize>) -> impl Iterator<Item = Self> {
		range.map(move |port| self.set_port(port))
	}

	pub fn node(self) -> Index {
		self.node
	}

	pub fn port(self) -> usize {
		self.port
	}
}

impl Default for Edge {
	fn default() -> Self {
		Self {
			node: Index::from_raw_parts(usize::MAX, u64::MAX),
			port: usize::MAX,
		}
	}
}

pub trait ToArena {
	fn to_arena(self, arena: &mut Arena<Node>) -> Edge;
}

impl<T> ToArena for T
where
	T: Into<Node>,
{
	fn to_arena(self, arena: &mut Arena<Node>) -> Edge {
		let index = arena.insert(self.into());

		Edge {
			node: index,
			port: 0,
		}
	}
}

#[derive(Clone)]
pub struct Undefined;

impl From<Undefined> for Node {
	fn from(undefined: Undefined) -> Self {
		Self::Simple(Simple::Undefined(undefined))
	}
}

pub enum Ordering {
	Memory,
	Global,
}

impl From<Ordering> for Node {
	fn from(ordering: Ordering) -> Self {
		Self::Simple(Simple::Ordering(ordering))
	}
}

// To ensure `Unreachable` has absolute ordering it takes all state edges
// as its input and becomes the new state.
pub struct Unreachable {
	pub(crate) memory_order: Edge,
	pub(crate) global_order: Edge,
}

impl From<Unreachable> for Node {
	fn from(unreachable: Unreachable) -> Self {
		Self::Simple(Simple::Unreachable(unreachable))
	}
}

pub struct Argument;

impl From<Argument> for Node {
	fn from(argument: Argument) -> Self {
		Self::Simple(Simple::Argument(argument))
	}
}

#[derive(Clone)]
pub enum Number {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

impl Number {
	pub fn zero_of(ty: ValType) -> Self {
		match ty {
			ValType::I32 => Self::I32(0),
			ValType::I64 => Self::I64(0),
			ValType::F32 => Self::F32(0.0),
			ValType::F64 => Self::F64(0.0),
			_ => unimplemented!(),
		}
	}
}

impl From<i32> for Number {
	fn from(i: i32) -> Self {
		Self::I32(i)
	}
}

impl From<i64> for Number {
	fn from(i: i64) -> Self {
		Self::I64(i)
	}
}

impl From<u32> for Number {
	fn from(i: u32) -> Self {
		Self::F32(f32::from_bits(i))
	}
}

impl From<u64> for Number {
	fn from(i: u64) -> Self {
		Self::F64(f64::from_bits(i))
	}
}

impl From<Number> for Node {
	fn from(number: Number) -> Self {
		Self::Simple(Simple::Number(number))
	}
}

pub struct GetFunction {
	pub(crate) function: usize,
}

impl From<GetFunction> for Node {
	fn from(get_function: GetFunction) -> Self {
		Self::Simple(Simple::GetFunction(get_function))
	}
}

pub struct GetTableElement {
	pub(crate) table: usize,
	pub(crate) index: Edge,
}

impl From<GetTableElement> for Node {
	fn from(get_table_element: GetTableElement) -> Self {
		Self::Simple(Simple::GetTableElement(get_table_element))
	}
}

pub struct Call {
	pub(crate) function: Edge,
	pub(crate) argument_list: Vec<Edge>,

	pub(crate) memory_order: Edge,
	pub(crate) global_order: Edge,
}

impl From<Call> for Node {
	fn from(call: Call) -> Self {
		Self::Simple(Simple::Call(call))
	}
}

pub struct GlobalGet {
	pub(crate) global: usize,

	pub(crate) order: Edge,
}

impl From<GlobalGet> for Node {
	fn from(global_get: GlobalGet) -> Self {
		Self::Simple(Simple::GlobalGet(global_get))
	}
}

pub struct GlobalSet {
	pub(crate) global: usize,
	pub(crate) value: Edge,

	pub(crate) order: Edge,
}

impl From<GlobalSet> for Node {
	fn from(global_set: GlobalSet) -> Self {
		Self::Simple(Simple::GlobalSet(global_set))
	}
}

pub struct Load {
	pub(crate) load_type: LoadType,
	pub(crate) memory: usize,
	pub(crate) offset: u64,
	pub(crate) pointer: Edge,

	pub(crate) order: Edge,
}

impl From<Load> for Node {
	fn from(load: Load) -> Self {
		Self::Simple(Simple::Load(load))
	}
}

pub struct Store {
	pub(crate) store_type: StoreType,
	pub(crate) memory: usize,
	pub(crate) offset: u64,
	pub(crate) pointer: Edge,
	pub(crate) value: Edge,

	pub(crate) order: Edge,
}

impl From<Store> for Node {
	fn from(store: Store) -> Self {
		Self::Simple(Simple::Store(store))
	}
}

pub struct MemorySize {
	pub(crate) memory: usize,

	pub(crate) order: Edge,
}

impl From<MemorySize> for Node {
	fn from(memory_size: MemorySize) -> Self {
		Self::Simple(Simple::MemorySize(memory_size))
	}
}

pub struct MemoryGrow {
	pub(crate) memory: usize,
	pub(crate) delta: Edge,

	pub(crate) order: Edge,
}

impl From<MemoryGrow> for Node {
	fn from(memory_grow: MemoryGrow) -> Self {
		Self::Simple(Simple::MemoryGrow(memory_grow))
	}
}

pub struct UnOp {
	pub(crate) op_type: UnOpType,
	pub(crate) rhs: Edge,
}

impl From<UnOp> for Node {
	fn from(unop: UnOp) -> Self {
		Self::Simple(Simple::UnOp(unop))
	}
}

pub struct BinOp {
	pub(crate) op_type: BinOpType,
	pub(crate) lhs: Edge,
	pub(crate) rhs: Edge,
}

impl From<BinOp> for Node {
	fn from(binop: BinOp) -> Self {
		Self::Simple(Simple::BinOp(binop))
	}
}

pub struct CmpOp {
	pub(crate) op_type: CmpOpType,
	pub(crate) lhs: Edge,
	pub(crate) rhs: Edge,
}

impl From<CmpOp> for Node {
	fn from(cmpop: CmpOp) -> Self {
		Self::Simple(Simple::CmpOp(cmpop))
	}
}

pub struct CastOp {
	pub(crate) op_type: CastOpType,
	pub(crate) rhs: Edge,
}

impl From<CastOp> for Node {
	fn from(castop: CastOp) -> Self {
		Self::Simple(Simple::CastOp(castop))
	}
}

pub enum Simple {
	Undefined(Undefined),
	Ordering(Ordering),
	Unreachable(Unreachable),

	Argument(Argument),
	Number(Number),

	GetFunction(GetFunction),
	GetTableElement(GetTableElement),
	Call(Call),

	GlobalGet(GlobalGet),
	GlobalSet(GlobalSet),

	Load(Load),
	Store(Store),

	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),

	UnOp(UnOp),
	BinOp(BinOp),
	CmpOp(CmpOp),
	CastOp(CastOp),
}

#[derive(Default)]
pub struct EdgeList {
	pub(crate) data: Vec<Edge>,
}

impl From<Vec<Edge>> for EdgeList {
	fn from(data: Vec<Edge>) -> Self {
		Self { data }
	}
}

impl FromIterator<Edge> for EdgeList {
	fn from_iter<I: IntoIterator<Item = Edge>>(iter: I) -> Self {
		let data = iter.into_iter().collect();

		Self { data }
	}
}

#[derive(Default)]
pub struct Gamma {
	pub(crate) condition: Edge,
	pub(crate) list_in: EdgeList,
	pub(crate) list_out: Vec<(Edge, EdgeList)>,
}

impl From<Gamma> for Node {
	fn from(gamma: Gamma) -> Self {
		Self::Gamma(gamma)
	}
}

#[derive(Default)]
pub struct Theta {
	pub(crate) param_edge: Edge,
	pub(crate) list_in: EdgeList,
	pub(crate) list_out: EdgeList,
	pub(crate) condition: Edge,
}

impl From<Theta> for Node {
	fn from(theta: Theta) -> Self {
		Self::Theta(theta)
	}
}

pub enum Node {
	Simple(Simple),
	Gamma(Gamma),
	Theta(Theta),
}

pub struct Lambda {
	pub(crate) arena: Arena<Node>,
	pub(crate) param_edge: Edge,
	pub(crate) list_out: EdgeList,
}
