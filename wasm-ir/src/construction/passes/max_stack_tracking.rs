use wasmparser::Operator;

use crate::module::TypeInfo;

pub struct MaxStackTracking<'t> {
	type_info: &'t TypeInfo<'t>,
}

impl<'t> MaxStackTracking<'t> {
	fn track(&self, op: &Operator) -> usize {
		match *op {
			Operator::Block { ty } | Operator::Loop { ty } | Operator::If { ty } => {
				self.type_info.by_block_type(ty).1
			}
			_ => 0,
		}
	}

	pub fn run(&self, code: &[Operator], result_count: usize) -> usize {
		let op_iter = code.iter().map(|v| self.track(v));
		let min_iter = std::iter::once(result_count);

		min_iter.chain(op_iter).max().unwrap()
	}
}

impl<'t> From<&'t TypeInfo<'t>> for MaxStackTracking<'t> {
	fn from(type_info: &'t TypeInfo<'t>) -> Self {
		Self { type_info }
	}
}
