use std::collections::BTreeSet;

use parity_wasm::elements::Instruction;

use crate::data::Module;

pub fn visit(m: &Module, index: usize) -> BTreeSet<u8> {
	let mut result = BTreeSet::new();

	for i in m.code[index].inst_list {
		match i {
			Instruction::I32Store(_, _)
			| Instruction::I64Store(_, _)
			| Instruction::F32Store(_, _)
			| Instruction::F64Store(_, _)
			| Instruction::I32Store8(_, _)
			| Instruction::I32Store16(_, _)
			| Instruction::I64Store8(_, _)
			| Instruction::I64Store16(_, _)
			| Instruction::I64Store32(_, _)
			| Instruction::I32Load(_, _)
			| Instruction::I64Load(_, _)
			| Instruction::F32Load(_, _)
			| Instruction::F64Load(_, _)
			| Instruction::I32Load8S(_, _)
			| Instruction::I32Load8U(_, _)
			| Instruction::I32Load16S(_, _)
			| Instruction::I32Load16U(_, _)
			| Instruction::I64Load8S(_, _)
			| Instruction::I64Load8U(_, _)
			| Instruction::I64Load16S(_, _)
			| Instruction::I64Load16U(_, _)
			| Instruction::I64Load32S(_, _)
			| Instruction::I64Load32U(_, _) => {
				result.insert(0);
			}
			Instruction::CurrentMemory(index) | Instruction::GrowMemory(index) => {
				result.insert(*index);
			}
			_ => {}
		}
	}

	result
}
