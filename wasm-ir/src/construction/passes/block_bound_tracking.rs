use std::collections::HashMap;

use wasmparser::Operator;

#[derive(Clone, Copy)]
pub struct Bound {
	pub start: usize,
	pub end: usize,
}

#[derive(PartialEq, Eq, Default)]
enum BoundType {
	#[default]
	Start,
	End,
	Code,
	Branch,
}

#[derive(Default)]
pub struct BlockBoundTracking {
	bound_map: HashMap<usize, Bound>,
	pending_list: Vec<usize>,
	last_id: usize,
	bound_type: BoundType,
}

impl BlockBoundTracking {
	fn set_bound_to(&mut self, typ: BoundType) {
		if self.bound_type != BoundType::Code && typ != self.bound_type {
			self.last_id += 1;
		}

		self.bound_type = typ;
	}

	fn add_block(&mut self, index: usize) {
		let bound = Bound {
			start: self.last_id,
			end: 0,
		};

		self.bound_map.insert(index, bound);
		self.pending_list.push(index);

		self.set_bound_to(BoundType::Start);
	}

	fn set_block_end(&mut self) {
		let last = self.pending_list.pop().unwrap();
		let bound = self.bound_map.get_mut(&last).unwrap();

		bound.end = self.last_id;

		self.set_bound_to(BoundType::End);
	}

	fn run_tracking(&mut self, code: &[Operator]) {
		self.bound_type = BoundType::Start;

		for (i, inst) in code.iter().enumerate() {
			match inst {
				Operator::Block { .. } | Operator::Loop { .. } | Operator::If { .. } => {
					self.add_block(i);
				}
				Operator::Else => {
					self.set_block_end();
					self.add_block(i);
				}
				Operator::End => {
					self.set_block_end();
				}
				Operator::BrIf { .. } => {
					self.set_bound_to(BoundType::Branch);
				}
				_ => {
					self.set_bound_to(BoundType::Code);
				}
			}
		}
	}

	pub fn run(&mut self, code: &[Operator]) -> HashMap<usize, Bound> {
		self.add_block(0);
		self.run_tracking(code);

		std::mem::take(&mut self.bound_map)
	}
}
