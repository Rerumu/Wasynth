use std::{collections::BTreeSet, convert::TryFrom};

use crate::{
	backend::helper::operation::{BinOp, Load, Named, Store, UnOp},
	data::Module,
};

pub fn visit(m: &Module, index: usize) -> BTreeSet<(&'static str, &'static str)> {
	let mut result = BTreeSet::new();

	for i in m.code[index].inst_list {
		if let Ok(used) = Load::try_from(i) {
			result.insert(used.as_name());
		} else if let Ok(used) = Store::try_from(i) {
			result.insert(used.as_name());
		} else if let Ok(used) = UnOp::try_from(i) {
			result.insert(used.as_name());
		} else if let Ok(used) = BinOp::try_from(i) {
			result.insert(used.as_name());
		}
	}

	result
}
