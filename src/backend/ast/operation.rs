use std::convert::TryFrom;

use parity_wasm::elements::Instruction;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Load {
	I32,
	I64,
	F32,
	F64,
	I32_I8,
	I32_U8,
	I32_I16,
	I32_U16,
	I64_I8,
	I64_U8,
	I64_I16,
	I64_U16,
	I64_I32,
	I64_U32,
}

impl Load {
	pub fn as_name(self) -> &'static str {
		match self {
			Self::I32 => "i32",
			Self::I64 => "i64",
			Self::F32 => "f32",
			Self::F64 => "f64",
			Self::I32_I8 => "i32_i8",
			Self::I32_U8 => "i32_u8",
			Self::I32_I16 => "i32_i16",
			Self::I32_U16 => "i32_u16",
			Self::I64_I8 => "i64_i8",
			Self::I64_U8 => "i64_u8",
			Self::I64_I16 => "i64_i16",
			Self::I64_U16 => "i64_u16",
			Self::I64_I32 => "i64_i32",
			Self::I64_U32 => "i64_u32",
		}
	}
}

impl TryFrom<&Instruction> for Load {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		let result = match inst {
			Instruction::I32Load(_, _) => Self::I32,
			Instruction::I64Load(_, _) => Self::I64,
			Instruction::F32Load(_, _) => Self::F32,
			Instruction::F64Load(_, _) => Self::F64,
			Instruction::I32Load8S(_, _) => Self::I32_I8,
			Instruction::I32Load8U(_, _) => Self::I32_U8,
			Instruction::I32Load16S(_, _) => Self::I32_I16,
			Instruction::I32Load16U(_, _) => Self::I32_U16,
			Instruction::I64Load8S(_, _) => Self::I64_I8,
			Instruction::I64Load8U(_, _) => Self::I64_U8,
			Instruction::I64Load16S(_, _) => Self::I64_I16,
			Instruction::I64Load16U(_, _) => Self::I64_U16,
			Instruction::I64Load32S(_, _) => Self::I64_I32,
			Instruction::I64Load32U(_, _) => Self::I64_U32,
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum Store {
	I32,
	I64,
	F32,
	F64,
	I32_N8,
	I32_N16,
	I64_N8,
	I64_N16,
	I64_N32,
}

impl Store {
	pub fn as_name(self) -> &'static str {
		match self {
			Self::I32 => "i32",
			Self::I64 => "i64",
			Self::F32 => "f32",
			Self::F64 => "f64",
			Self::I32_N8 => "i32_n8",
			Self::I32_N16 => "i32_n16",
			Self::I64_N8 => "i64_n8",
			Self::I64_N16 => "i64_n16",
			Self::I64_N32 => "i64_n32",
		}
	}
}

impl TryFrom<&Instruction> for Store {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		let result = match inst {
			Instruction::I32Store(_, _) => Self::I32,
			Instruction::I64Store(_, _) => Self::I64,
			Instruction::F32Store(_, _) => Self::F32,
			Instruction::F64Store(_, _) => Self::F64,
			Instruction::I32Store8(_, _) => Self::I32_N8,
			Instruction::I32Store16(_, _) => Self::I32_N16,
			Instruction::I64Store8(_, _) => Self::I64_N8,
			Instruction::I64Store16(_, _) => Self::I64_N16,
			Instruction::I64Store32(_, _) => Self::I64_N32,
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum UnOp {
	Eqz_I32,
	Eqz_I64,
	Clz_I32,
	Ctz_I32,
	Popcnt_I32,
	Clz_I64,
	Ctz_I64,
	Popcnt_I64,
	Abs_FN,
	Neg_FN,
	Ceil_FN,
	Floor_FN,
	Trunc_FN,
	Nearest_FN,
	Sqrt_FN,
	Copysign_FN,
	Wrap_I32_I64,
	Trunc_I32_F32,
	Trunc_U32_F32,
	Trunc_I32_F64,
	Trunc_U32_F64,
	Extend_I64_I32,
	Extend_U64_I32,
	Trunc_I64_F32,
	Trunc_U64_F32,
	Trunc_I64_F64,
	Trunc_U64_F64,
	Convert_F32_I32,
	Convert_F32_U32,
	Convert_F32_I64,
	Convert_F32_U64,
	Demote_F32_F64,
	Convert_F64_I32,
	Convert_F64_U32,
	Convert_F64_I64,
	Convert_F64_U64,
	Promote_F64_F32,
	Reinterpret_I32_F32,
	Reinterpret_I64_F64,
	Reinterpret_F32_I32,
	Reinterpret_F64_I64,
}

impl UnOp {
	pub fn as_operator(self) -> Option<&'static str> {
		let op = match self {
			Self::Neg_FN => "-",
			_ => return None,
		};

		Some(op)
	}

	pub fn as_name(self) -> (&'static str, &'static str) {
		match self {
			Self::Eqz_I32 => ("eqz", "i32"),
			Self::Eqz_I64 => ("eqz", "i64"),
			Self::Clz_I32 => ("clz", "i32"),
			Self::Ctz_I32 => ("ctz", "i32"),
			Self::Popcnt_I32 => ("popcnt", "i32"),
			Self::Clz_I64 => ("clz", "i64"),
			Self::Ctz_I64 => ("ctz", "i64"),
			Self::Popcnt_I64 => ("popcnt", "i64"),
			Self::Abs_FN => ("math", "abs"),
			Self::Neg_FN => ("neg", "num"),
			Self::Ceil_FN => ("math", "ceil"),
			Self::Floor_FN => ("math", "floor"),
			Self::Trunc_FN => ("trunc", "num"),
			Self::Nearest_FN => ("nearest", "num"),
			Self::Sqrt_FN => ("math", "sqrt"),
			Self::Copysign_FN => ("math", "sign"),
			Self::Wrap_I32_I64 => ("wrap", "i64_i32"),
			Self::Trunc_I32_F32 => ("trunc", "f32_i32"),
			Self::Trunc_U32_F32 => ("trunc", "f32_u32"),
			Self::Trunc_I32_F64 => ("trunc", "f64_i32"),
			Self::Trunc_U32_F64 => ("trunc", "f64_u32"),
			Self::Extend_I64_I32 => ("extend", "i32_i64"),
			Self::Extend_U64_I32 => ("extend", "i32_u64"),
			Self::Trunc_I64_F32 => ("trunc", "f32_i64"),
			Self::Trunc_U64_F32 => ("trunc", "f32_u64"),
			Self::Trunc_I64_F64 => ("trunc", "f64_i64"),
			Self::Trunc_U64_F64 => ("trunc", "f64_u64"),
			Self::Convert_F32_I32 => ("convert", "i32_f32"),
			Self::Convert_F32_U32 => ("convert", "u32_f32"),
			Self::Convert_F32_I64 => ("convert", "i64_f32"),
			Self::Convert_F32_U64 => ("convert", "u64_f32"),
			Self::Demote_F32_F64 => ("demote", "f64_f32"),
			Self::Convert_F64_I32 => ("convert", "f64_i32"),
			Self::Convert_F64_U32 => ("convert", "f64_u32"),
			Self::Convert_F64_I64 => ("convert", "f64_i64"),
			Self::Convert_F64_U64 => ("convert", "f64_u64"),
			Self::Promote_F64_F32 => ("promote", "f32_f64"),
			Self::Reinterpret_I32_F32 => ("reinterpret", "f32_i32"),
			Self::Reinterpret_I64_F64 => ("reinterpret", "f64_i64"),
			Self::Reinterpret_F32_I32 => ("reinterpret", "i32_f32"),
			Self::Reinterpret_F64_I64 => ("reinterpret", "i64_f64"),
		}
	}
}

impl TryFrom<&Instruction> for UnOp {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		let result = match inst {
			Instruction::I32Eqz => Self::Eqz_I32,
			Instruction::I64Eqz => Self::Eqz_I64,
			Instruction::I32Clz => Self::Clz_I32,
			Instruction::I32Ctz => Self::Ctz_I32,
			Instruction::I32Popcnt => Self::Popcnt_I32,
			Instruction::I64Clz => Self::Clz_I64,
			Instruction::I64Ctz => Self::Ctz_I64,
			Instruction::I64Popcnt => Self::Popcnt_I64,
			Instruction::F32Abs | Instruction::F64Abs => Self::Abs_FN,
			Instruction::F32Neg | Instruction::F64Neg => Self::Neg_FN,
			Instruction::F32Ceil | Instruction::F64Ceil => Self::Ceil_FN,
			Instruction::F32Floor | Instruction::F64Floor => Self::Floor_FN,
			Instruction::F32Trunc | Instruction::F64Trunc => Self::Trunc_FN,
			Instruction::F32Nearest | Instruction::F64Nearest => Self::Nearest_FN,
			Instruction::F32Sqrt | Instruction::F64Sqrt => Self::Sqrt_FN,
			Instruction::F32Copysign | Instruction::F64Copysign => Self::Copysign_FN,
			Instruction::I32WrapI64 => Self::Wrap_I32_I64,
			Instruction::I32TruncSF32 => Self::Trunc_I32_F32,
			Instruction::I32TruncUF32 => Self::Trunc_U32_F32,
			Instruction::I32TruncSF64 => Self::Trunc_I32_F64,
			Instruction::I32TruncUF64 => Self::Trunc_U32_F64,
			Instruction::I64ExtendSI32 => Self::Extend_I64_I32,
			Instruction::I64ExtendUI32 => Self::Extend_U64_I32,
			Instruction::I64TruncSF32 => Self::Trunc_I64_F32,
			Instruction::I64TruncUF32 => Self::Trunc_U64_F32,
			Instruction::I64TruncSF64 => Self::Trunc_I64_F64,
			Instruction::I64TruncUF64 => Self::Trunc_U64_F64,
			Instruction::F32ConvertSI32 => Self::Convert_F32_I32,
			Instruction::F32ConvertUI32 => Self::Convert_F32_U32,
			Instruction::F32ConvertSI64 => Self::Convert_F32_I64,
			Instruction::F32ConvertUI64 => Self::Convert_F32_U64,
			Instruction::F32DemoteF64 => Self::Demote_F32_F64,
			Instruction::F64ConvertSI32 => Self::Convert_F64_I32,
			Instruction::F64ConvertUI32 => Self::Convert_F64_U32,
			Instruction::F64ConvertSI64 => Self::Convert_F64_I64,
			Instruction::F64ConvertUI64 => Self::Convert_F64_U64,
			Instruction::F64PromoteF32 => Self::Promote_F64_F32,
			Instruction::I32ReinterpretF32 => Self::Reinterpret_I32_F32,
			Instruction::I64ReinterpretF64 => Self::Reinterpret_I64_F64,
			Instruction::F32ReinterpretI32 => Self::Reinterpret_F32_I32,
			Instruction::F64ReinterpretI64 => Self::Reinterpret_F64_I64,
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum BinOp {
	Eq_I32,
	Ne_I32,
	LtS_I32,
	LtU_I32,
	GtS_I32,
	GtU_I32,
	LeS_I32,
	LeU_I32,
	GeS_I32,
	GeU_I32,
	Eq_I64,
	Ne_I64,
	LtS_I64,
	LtU_I64,
	GtS_I64,
	GtU_I64,
	LeS_I64,
	LeU_I64,
	GeS_I64,
	GeU_I64,
	Add_I32,
	Sub_I32,
	Mul_I32,
	DivS_I32,
	DivU_I32,
	RemS_I32,
	RemU_I32,
	And_I32,
	Or_I32,
	Xor_I32,
	Shl_I32,
	ShrS_I32,
	ShrU_I32,
	Rotl_I32,
	Rotr_I32,
	Add_I64,
	Sub_I64,
	Mul_I64,
	DivS_I64,
	DivU_I64,
	RemS_I64,
	RemU_I64,
	And_I64,
	Or_I64,
	Xor_I64,
	Shl_I64,
	ShrS_I64,
	ShrU_I64,
	Rotl_I64,
	Rotr_I64,
	Eq_FN,
	Ne_FN,
	Lt_FN,
	Gt_FN,
	Le_FN,
	Ge_FN,
	Add_FN,
	Sub_FN,
	Mul_FN,
	Div_FN,
	Min_FN,
	Max_FN,
}

impl BinOp {
	pub fn as_operator(self) -> Option<&'static str> {
		let op = match self {
			Self::Add_I32 | Self::Add_I64 | Self::Add_FN => "+",
			Self::Sub_I32 | Self::Sub_I64 | Self::Sub_FN => "-",
			Self::Mul_I32 | Self::Mul_I64 | Self::Mul_FN => "*",
			Self::Div_FN => "/",
			Self::RemS_I32 | Self::RemU_I32 | Self::RemS_I64 | Self::RemU_I64 => "%",
			_ => return None,
		};

		Some(op)
	}

	pub fn as_name(self) -> (&'static str, &'static str) {
		match self {
			Self::Eq_I32 => ("eq", "i32"),
			Self::Ne_I32 => ("ne", "i32"),
			Self::LtS_I32 => ("lt", "i32"),
			Self::LtU_I32 => ("lt", "u32"),
			Self::GtS_I32 => ("gt", "i32"),
			Self::GtU_I32 => ("gt", "u32"),
			Self::LeS_I32 => ("le", "i32"),
			Self::LeU_I32 => ("le", "u32"),
			Self::GeS_I32 => ("ge", "i32"),
			Self::GeU_I32 => ("ge", "u32"),
			Self::Eq_I64 => ("eq", "i64"),
			Self::Ne_I64 => ("ne", "i64"),
			Self::LtS_I64 => ("lt", "i64"),
			Self::LtU_I64 => ("lt", "u64"),
			Self::GtS_I64 => ("gt", "i64"),
			Self::GtU_I64 => ("gt", "u64"),
			Self::LeS_I64 => ("le", "i64"),
			Self::LeU_I64 => ("le", "u64"),
			Self::GeS_I64 => ("ge", "i64"),
			Self::GeU_I64 => ("ge", "u64"),
			Self::Add_I32 => ("add", "i32"),
			Self::Sub_I32 => ("sub", "i32"),
			Self::Mul_I32 => ("mul", "i32"),
			Self::DivS_I32 => ("div", "i32"),
			Self::DivU_I32 => ("div", "u32"),
			Self::RemS_I32 => ("rem", "i32"),
			Self::RemU_I32 => ("rem", "u32"),
			Self::And_I32 => ("band", "i32"),
			Self::Or_I32 => ("bor", "i32"),
			Self::Xor_I32 => ("bxor", "i32"),
			Self::Shl_I32 => ("shl", "i32"),
			Self::ShrS_I32 => ("shr", "i32"),
			Self::ShrU_I32 => ("shr", "u32"),
			Self::Rotl_I32 => ("rotl", "i32"),
			Self::Rotr_I32 => ("rotr", "i32"),
			Self::Add_I64 => ("add", "i64"),
			Self::Sub_I64 => ("sub", "i64"),
			Self::Mul_I64 => ("mul", "i64"),
			Self::DivS_I64 => ("div", "i64"),
			Self::DivU_I64 => ("div", "u64"),
			Self::RemS_I64 => ("rem", "i64"),
			Self::RemU_I64 => ("rem", "u64"),
			Self::And_I64 => ("band", "i64"),
			Self::Or_I64 => ("bor", "i64"),
			Self::Xor_I64 => ("bxor", "i64"),
			Self::Shl_I64 => ("shl", "i64"),
			Self::ShrS_I64 => ("shr", "i64"),
			Self::ShrU_I64 => ("shr", "u64"),
			Self::Rotl_I64 => ("rotl", "i64"),
			Self::Rotr_I64 => ("rotr", "i64"),
			Self::Eq_FN => ("eq", "num"),
			Self::Ne_FN => ("ne", "num"),
			Self::Lt_FN => ("lt", "num"),
			Self::Gt_FN => ("gt", "num"),
			Self::Le_FN => ("le", "num"),
			Self::Ge_FN => ("ge", "num"),
			Self::Add_FN => ("add", "num"),
			Self::Sub_FN => ("sub", "num"),
			Self::Mul_FN => ("mul", "num"),
			Self::Div_FN => ("div", "num"),
			Self::Min_FN => ("math", "min"),
			Self::Max_FN => ("math", "max"),
		}
	}
}

impl TryFrom<&Instruction> for BinOp {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		let result = match inst {
			Instruction::I32Eq => Self::Eq_I32,
			Instruction::I32Ne => Self::Ne_I32,
			Instruction::I32LtS => Self::LtS_I32,
			Instruction::I32LtU => Self::LtU_I32,
			Instruction::I32GtS => Self::GtS_I32,
			Instruction::I32GtU => Self::GtU_I32,
			Instruction::I32LeS => Self::LeS_I32,
			Instruction::I32LeU => Self::LeU_I32,
			Instruction::I32GeS => Self::GeS_I32,
			Instruction::I32GeU => Self::GeU_I32,
			Instruction::I64Eq => Self::Eq_I64,
			Instruction::I64Ne => Self::Ne_I64,
			Instruction::I64LtS => Self::LtS_I64,
			Instruction::I64LtU => Self::LtU_I64,
			Instruction::I64GtS => Self::GtS_I64,
			Instruction::I64GtU => Self::GtU_I64,
			Instruction::I64LeS => Self::LeS_I64,
			Instruction::I64LeU => Self::LeU_I64,
			Instruction::I64GeS => Self::GeS_I64,
			Instruction::I64GeU => Self::GeU_I64,
			Instruction::I32Add => Self::Add_I32,
			Instruction::I32Sub => Self::Sub_I32,
			Instruction::I32Mul => Self::Mul_I32,
			Instruction::I32DivS => Self::DivS_I32,
			Instruction::I32DivU => Self::DivU_I32,
			Instruction::I32RemS => Self::RemS_I32,
			Instruction::I32RemU => Self::RemU_I32,
			Instruction::I32And => Self::And_I32,
			Instruction::I32Or => Self::Or_I32,
			Instruction::I32Xor => Self::Xor_I32,
			Instruction::I32Shl => Self::Shl_I32,
			Instruction::I32ShrS => Self::ShrS_I32,
			Instruction::I32ShrU => Self::ShrU_I32,
			Instruction::I32Rotl => Self::Rotl_I32,
			Instruction::I32Rotr => Self::Rotr_I32,
			Instruction::I64Add => Self::Add_I64,
			Instruction::I64Sub => Self::Sub_I64,
			Instruction::I64Mul => Self::Mul_I64,
			Instruction::I64DivS => Self::DivS_I64,
			Instruction::I64DivU => Self::DivU_I64,
			Instruction::I64RemS => Self::RemS_I64,
			Instruction::I64RemU => Self::RemU_I64,
			Instruction::I64And => Self::And_I64,
			Instruction::I64Or => Self::Or_I64,
			Instruction::I64Xor => Self::Xor_I64,
			Instruction::I64Shl => Self::Shl_I64,
			Instruction::I64ShrS => Self::ShrS_I64,
			Instruction::I64ShrU => Self::ShrU_I64,
			Instruction::I64Rotl => Self::Rotl_I64,
			Instruction::I64Rotr => Self::Rotr_I64,
			Instruction::F32Eq | Instruction::F64Eq => Self::Eq_FN,
			Instruction::F32Ne | Instruction::F64Ne => Self::Ne_FN,
			Instruction::F32Lt | Instruction::F64Lt => Self::Lt_FN,
			Instruction::F32Gt | Instruction::F64Gt => Self::Gt_FN,
			Instruction::F32Le | Instruction::F64Le => Self::Le_FN,
			Instruction::F32Ge | Instruction::F64Ge => Self::Ge_FN,
			Instruction::F32Add | Instruction::F64Add => Self::Add_FN,
			Instruction::F32Sub | Instruction::F64Sub => Self::Sub_FN,
			Instruction::F32Mul | Instruction::F64Mul => Self::Mul_FN,
			Instruction::F32Div | Instruction::F64Div => Self::Div_FN,
			Instruction::F32Min | Instruction::F64Min => Self::Min_FN,
			Instruction::F32Max | Instruction::F64Max => Self::Max_FN,
			_ => {
				return Err(());
			}
		};

		Ok(result)
	}
}
