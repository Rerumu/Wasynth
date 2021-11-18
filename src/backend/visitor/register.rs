use parity_wasm::elements::Instruction;

use crate::{
	backend::helper::register::Register,
	data::{Arity, Module},
};

pub fn visit(m: &Module, index: usize) -> u32 {
	let mut reg = Register::new();
	let num_param = m.in_arity[index].num_param;
	let num_local = m.code[index].num_local;

	reg.push(num_param + num_local);

	for i in m.code[index].inst_list {
		match i {
			Instruction::Block(_) | Instruction::Loop(_) => {
				reg.save();
			}
			Instruction::If(_) => {
				reg.pop(1);
				reg.save();
			}
			Instruction::Else => {
				reg.load();
				reg.save();
			}
			Instruction::End => {
				reg.load();
			}
			Instruction::BrIf(_)
			| Instruction::BrTable(_)
			| Instruction::Drop
			| Instruction::SetLocal(_)
			| Instruction::SetGlobal(_) => {
				reg.pop(1);
			}
			Instruction::Call(i) => {
				let arity = m.arity_of(*i as usize);

				reg.pop(arity.num_param);
				reg.push(arity.num_result);
			}
			Instruction::CallIndirect(i, _) => {
				let types = m.parent.type_section().unwrap().types();
				let arity = Arity::from_index(types, *i);

				reg.pop(arity.num_param + 1);
				reg.push(arity.num_result);
			}
			Instruction::Select => {
				reg.pop(3);
				reg.push(1);
			}
			Instruction::GetLocal(_)
			| Instruction::GetGlobal(_)
			| Instruction::CurrentMemory(_)
			| Instruction::I32Const(_)
			| Instruction::I64Const(_)
			| Instruction::F32Const(_)
			| Instruction::F64Const(_) => {
				reg.push(1);
			}
			Instruction::TeeLocal(_)
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
			| Instruction::I64Load32U(_, _)
			| Instruction::GrowMemory(_)
			| Instruction::I32Eqz
			| Instruction::I64Eqz
			| Instruction::I32Clz
			| Instruction::I32Ctz
			| Instruction::I32Popcnt
			| Instruction::I64Clz
			| Instruction::I64Ctz
			| Instruction::I64Popcnt
			| Instruction::F32Abs
			| Instruction::F32Neg
			| Instruction::F32Ceil
			| Instruction::F32Floor
			| Instruction::F32Trunc
			| Instruction::F32Nearest
			| Instruction::F32Sqrt
			| Instruction::F32Copysign
			| Instruction::F64Abs
			| Instruction::F64Neg
			| Instruction::F64Ceil
			| Instruction::F64Floor
			| Instruction::F64Trunc
			| Instruction::F64Nearest
			| Instruction::F64Sqrt
			| Instruction::F64Copysign
			| Instruction::I32WrapI64
			| Instruction::I32TruncSF32
			| Instruction::I32TruncUF32
			| Instruction::I32TruncSF64
			| Instruction::I32TruncUF64
			| Instruction::I64ExtendSI32
			| Instruction::I64ExtendUI32
			| Instruction::I64TruncSF32
			| Instruction::I64TruncUF32
			| Instruction::I64TruncSF64
			| Instruction::I64TruncUF64
			| Instruction::F32ConvertSI32
			| Instruction::F32ConvertUI32
			| Instruction::F32ConvertSI64
			| Instruction::F32ConvertUI64
			| Instruction::F32DemoteF64
			| Instruction::F64ConvertSI32
			| Instruction::F64ConvertUI32
			| Instruction::F64ConvertSI64
			| Instruction::F64ConvertUI64
			| Instruction::F64PromoteF32
			| Instruction::I32ReinterpretF32
			| Instruction::I64ReinterpretF64
			| Instruction::F32ReinterpretI32
			| Instruction::F64ReinterpretI64 => {
				reg.pop(1);
				reg.push(1);
			}
			Instruction::I32Store(_, _)
			| Instruction::I64Store(_, _)
			| Instruction::F32Store(_, _)
			| Instruction::F64Store(_, _)
			| Instruction::I32Store8(_, _)
			| Instruction::I32Store16(_, _)
			| Instruction::I64Store8(_, _)
			| Instruction::I64Store16(_, _)
			| Instruction::I64Store32(_, _) => {
				reg.pop(2);
			}
			Instruction::I32Eq
			| Instruction::I32Ne
			| Instruction::I32LtS
			| Instruction::I32LtU
			| Instruction::I32GtS
			| Instruction::I32GtU
			| Instruction::I32LeS
			| Instruction::I32LeU
			| Instruction::I32GeS
			| Instruction::I32GeU
			| Instruction::I64Eq
			| Instruction::I64Ne
			| Instruction::I64LtS
			| Instruction::I64LtU
			| Instruction::I64GtS
			| Instruction::I64GtU
			| Instruction::I64LeS
			| Instruction::I64LeU
			| Instruction::I64GeS
			| Instruction::I64GeU
			| Instruction::F32Eq
			| Instruction::F32Ne
			| Instruction::F32Lt
			| Instruction::F32Gt
			| Instruction::F32Le
			| Instruction::F32Ge
			| Instruction::F64Eq
			| Instruction::F64Ne
			| Instruction::F64Lt
			| Instruction::F64Gt
			| Instruction::F64Le
			| Instruction::F64Ge
			| Instruction::I32Add
			| Instruction::I32Sub
			| Instruction::I32Mul
			| Instruction::I32DivS
			| Instruction::I32DivU
			| Instruction::I32RemS
			| Instruction::I32RemU
			| Instruction::I32And
			| Instruction::I32Or
			| Instruction::I32Xor
			| Instruction::I32Shl
			| Instruction::I32ShrS
			| Instruction::I32ShrU
			| Instruction::I32Rotl
			| Instruction::I32Rotr
			| Instruction::I64Add
			| Instruction::I64Sub
			| Instruction::I64Mul
			| Instruction::I64DivS
			| Instruction::I64DivU
			| Instruction::I64RemS
			| Instruction::I64RemU
			| Instruction::I64And
			| Instruction::I64Or
			| Instruction::I64Xor
			| Instruction::I64Shl
			| Instruction::I64ShrS
			| Instruction::I64ShrU
			| Instruction::I64Rotl
			| Instruction::I64Rotr
			| Instruction::F32Add
			| Instruction::F32Sub
			| Instruction::F32Mul
			| Instruction::F32Div
			| Instruction::F32Min
			| Instruction::F32Max
			| Instruction::F64Add
			| Instruction::F64Sub
			| Instruction::F64Mul
			| Instruction::F64Div
			| Instruction::F64Min
			| Instruction::F64Max => {
				reg.pop(2);
				reg.push(1);
			}
			_ => {}
		}
	}

	reg.last
}
