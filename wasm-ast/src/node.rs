use std::ops::Range;

use wasmparser::{Operator, ValType};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum LoadType {
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

impl LoadType {
	#[must_use]
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

impl TryFrom<&Operator<'_>> for LoadType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Load { .. } => Self::I32,
			Operator::I64Load { .. } => Self::I64,
			Operator::F32Load { .. } => Self::F32,
			Operator::F64Load { .. } => Self::F64,
			Operator::I32Load8S { .. } => Self::I32_I8,
			Operator::I32Load8U { .. } => Self::I32_U8,
			Operator::I32Load16S { .. } => Self::I32_I16,
			Operator::I32Load16U { .. } => Self::I32_U16,
			Operator::I64Load8S { .. } => Self::I64_I8,
			Operator::I64Load8U { .. } => Self::I64_U8,
			Operator::I64Load16S { .. } => Self::I64_I16,
			Operator::I64Load16U { .. } => Self::I64_U16,
			Operator::I64Load32S { .. } => Self::I64_I32,
			Operator::I64Load32U { .. } => Self::I64_U32,
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum StoreType {
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

impl StoreType {
	#[must_use]
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

impl TryFrom<&Operator<'_>> for StoreType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Store { .. } => Self::I32,
			Operator::I64Store { .. } => Self::I64,
			Operator::F32Store { .. } => Self::F32,
			Operator::F64Store { .. } => Self::F64,
			Operator::I32Store8 { .. } => Self::I32_N8,
			Operator::I32Store16 { .. } => Self::I32_N16,
			Operator::I64Store8 { .. } => Self::I64_N8,
			Operator::I64Store16 { .. } => Self::I64_N16,
			Operator::I64Store32 { .. } => Self::I64_N32,
			_ => return Err(()),
		};

		Ok(result)
	}
}

// Order of mnemonics is:
// operation_result_parameter
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum UnOpType {
	Clz_I32,
	Ctz_I32,
	Popcnt_I32,
	Clz_I64,
	Ctz_I64,
	Popcnt_I64,
	Abs_F32,
	Neg_F32,
	Ceil_F32,
	Floor_F32,
	Truncate_F32,
	Nearest_F32,
	Sqrt_F32,
	Abs_F64,
	Neg_F64,
	Ceil_F64,
	Floor_F64,
	Truncate_F64,
	Nearest_F64,
	Sqrt_F64,
	Wrap_I32_I64,
	Truncate_I32_F32,
	Truncate_I32_F64,
	Truncate_U32_F32,
	Truncate_U32_F64,
	Truncate_I64_F32,
	Truncate_I64_F64,
	Truncate_U64_F32,
	Truncate_U64_F64,
	Saturate_I32_F32,
	Saturate_I32_F64,
	Saturate_U32_F32,
	Saturate_U32_F64,
	Saturate_I64_F32,
	Saturate_I64_F64,
	Saturate_U64_F32,
	Saturate_U64_F64,
	Extend_I32_N8,
	Extend_I32_N16,
	Extend_I64_N8,
	Extend_I64_N16,
	Extend_I64_N32,
	Extend_I64_I32,
	Extend_I64_U32,
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

impl UnOpType {
	#[must_use]
	pub fn as_name(self) -> (&'static str, &'static str) {
		match self {
			Self::Clz_I32 => ("clz", "i32"),
			Self::Ctz_I32 => ("ctz", "i32"),
			Self::Popcnt_I32 => ("popcnt", "i32"),
			Self::Clz_I64 => ("clz", "i64"),
			Self::Ctz_I64 => ("ctz", "i64"),
			Self::Popcnt_I64 => ("popcnt", "i64"),
			Self::Abs_F32 => ("abs", "f32"),
			Self::Neg_F32 => ("neg", "f32"),
			Self::Ceil_F32 => ("ceil", "f32"),
			Self::Floor_F32 => ("floor", "f32"),
			Self::Truncate_F32 => ("truncate", "f32"),
			Self::Nearest_F32 => ("nearest", "f32"),
			Self::Sqrt_F32 => ("sqrt", "f32"),
			Self::Abs_F64 => ("abs", "f64"),
			Self::Neg_F64 => ("neg", "f64"),
			Self::Ceil_F64 => ("ceil", "f64"),
			Self::Floor_F64 => ("floor", "f64"),
			Self::Truncate_F64 => ("truncate", "f64"),
			Self::Nearest_F64 => ("nearest", "f64"),
			Self::Sqrt_F64 => ("sqrt", "f64"),
			Self::Wrap_I32_I64 => ("wrap", "i32_i64"),
			Self::Truncate_I32_F32 => ("truncate", "i32_f32"),
			Self::Truncate_I32_F64 => ("truncate", "i32_f64"),
			Self::Truncate_U32_F32 => ("truncate", "u32_f32"),
			Self::Truncate_U32_F64 => ("truncate", "u32_f64"),
			Self::Truncate_I64_F32 => ("truncate", "i64_f32"),
			Self::Truncate_I64_F64 => ("truncate", "i64_f64"),
			Self::Truncate_U64_F32 => ("truncate", "u64_f32"),
			Self::Truncate_U64_F64 => ("truncate", "u64_f64"),
			Self::Saturate_I32_F32 => ("saturate", "i32_f32"),
			Self::Saturate_I32_F64 => ("saturate", "i32_f64"),
			Self::Saturate_U32_F32 => ("saturate", "u32_f32"),
			Self::Saturate_U32_F64 => ("saturate", "u32_f64"),
			Self::Saturate_I64_F32 => ("saturate", "i64_f32"),
			Self::Saturate_I64_F64 => ("saturate", "i64_f64"),
			Self::Saturate_U64_F32 => ("saturate", "u64_f32"),
			Self::Saturate_U64_F64 => ("saturate", "u64_f64"),
			Self::Extend_I32_N8 => ("extend", "i32_n8"),
			Self::Extend_I32_N16 => ("extend", "i32_n16"),
			Self::Extend_I64_N8 => ("extend", "i64_n8"),
			Self::Extend_I64_N16 => ("extend", "i64_n16"),
			Self::Extend_I64_N32 => ("extend", "i64_n32"),
			Self::Extend_I64_I32 => ("extend", "i64_i32"),
			Self::Extend_I64_U32 => ("extend", "i64_u32"),
			Self::Convert_F32_I32 => ("convert", "f32_i32"),
			Self::Convert_F32_U32 => ("convert", "f32_u32"),
			Self::Convert_F32_I64 => ("convert", "f32_i64"),
			Self::Convert_F32_U64 => ("convert", "f32_u64"),
			Self::Demote_F32_F64 => ("demote", "f32_f64"),
			Self::Convert_F64_I32 => ("convert", "f64_i32"),
			Self::Convert_F64_U32 => ("convert", "f64_u32"),
			Self::Convert_F64_I64 => ("convert", "f64_i64"),
			Self::Convert_F64_U64 => ("convert", "f64_u64"),
			Self::Promote_F64_F32 => ("promote", "f64_f32"),
			Self::Reinterpret_I32_F32 => ("reinterpret", "i32_f32"),
			Self::Reinterpret_I64_F64 => ("reinterpret", "i64_f64"),
			Self::Reinterpret_F32_I32 => ("reinterpret", "f32_i32"),
			Self::Reinterpret_F64_I64 => ("reinterpret", "f64_i64"),
		}
	}
}

impl TryFrom<&Operator<'_>> for UnOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Clz => Self::Clz_I32,
			Operator::I32Ctz => Self::Ctz_I32,
			Operator::I32Popcnt => Self::Popcnt_I32,
			Operator::I64Clz => Self::Clz_I64,
			Operator::I64Ctz => Self::Ctz_I64,
			Operator::I64Popcnt => Self::Popcnt_I64,
			Operator::F32Abs => Self::Abs_F32,
			Operator::F32Neg => Self::Neg_F32,
			Operator::F32Ceil => Self::Ceil_F32,
			Operator::F32Floor => Self::Floor_F32,
			Operator::F32Trunc => Self::Truncate_F32,
			Operator::F32Nearest => Self::Nearest_F32,
			Operator::F32Sqrt => Self::Sqrt_F32,
			Operator::F64Abs => Self::Abs_F64,
			Operator::F64Neg => Self::Neg_F64,
			Operator::F64Ceil => Self::Ceil_F64,
			Operator::F64Floor => Self::Floor_F64,
			Operator::F64Trunc => Self::Truncate_F64,
			Operator::F64Nearest => Self::Nearest_F64,
			Operator::F64Sqrt => Self::Sqrt_F64,
			Operator::I32WrapI64 => Self::Wrap_I32_I64,
			Operator::I32TruncF32S => Self::Truncate_I32_F32,
			Operator::I32TruncF64S => Self::Truncate_I32_F64,
			Operator::I32TruncF32U => Self::Truncate_U32_F32,
			Operator::I32TruncF64U => Self::Truncate_U32_F64,
			Operator::I64TruncF32S => Self::Truncate_I64_F32,
			Operator::I64TruncF64S => Self::Truncate_I64_F64,
			Operator::I64TruncF32U => Self::Truncate_U64_F32,
			Operator::I64TruncF64U => Self::Truncate_U64_F64,
			Operator::I32TruncSatF32S => Self::Saturate_I32_F32,
			Operator::I32TruncSatF64S => Self::Saturate_I32_F64,
			Operator::I32TruncSatF32U => Self::Saturate_U32_F32,
			Operator::I32TruncSatF64U => Self::Saturate_U32_F64,
			Operator::I64TruncSatF32S => Self::Saturate_I64_F32,
			Operator::I64TruncSatF64S => Self::Saturate_I64_F64,
			Operator::I64TruncSatF32U => Self::Saturate_U64_F32,
			Operator::I64TruncSatF64U => Self::Saturate_U64_F64,
			Operator::I32Extend8S => Self::Extend_I32_N8,
			Operator::I32Extend16S => Self::Extend_I32_N16,
			Operator::I64Extend8S => Self::Extend_I64_N8,
			Operator::I64Extend16S => Self::Extend_I64_N16,
			Operator::I64Extend32S => Self::Extend_I64_N32,
			Operator::I64ExtendI32S => Self::Extend_I64_I32,
			Operator::I64ExtendI32U => Self::Extend_I64_U32,
			Operator::F32ConvertI32S => Self::Convert_F32_I32,
			Operator::F32ConvertI32U => Self::Convert_F32_U32,
			Operator::F32ConvertI64S => Self::Convert_F32_I64,
			Operator::F32ConvertI64U => Self::Convert_F32_U64,
			Operator::F32DemoteF64 => Self::Demote_F32_F64,
			Operator::F64ConvertI32S => Self::Convert_F64_I32,
			Operator::F64ConvertI32U => Self::Convert_F64_U32,
			Operator::F64ConvertI64S => Self::Convert_F64_I64,
			Operator::F64ConvertI64U => Self::Convert_F64_U64,
			Operator::F64PromoteF32 => Self::Promote_F64_F32,
			Operator::I32ReinterpretF32 => Self::Reinterpret_I32_F32,
			Operator::I64ReinterpretF64 => Self::Reinterpret_I64_F64,
			Operator::F32ReinterpretI32 => Self::Reinterpret_F32_I32,
			Operator::F64ReinterpretI64 => Self::Reinterpret_F64_I64,
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum BinOpType {
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
	Add_F32,
	Sub_F32,
	Mul_F32,
	Div_F32,
	Min_F32,
	Max_F32,
	Copysign_F32,
	Add_F64,
	Sub_F64,
	Mul_F64,
	Div_F64,
	Min_F64,
	Max_F64,
	Copysign_F64,
}

impl BinOpType {
	#[must_use]
	pub fn as_name(self) -> (&'static str, &'static str) {
		match self {
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
			Self::Add_F32 => ("add", "f32"),
			Self::Sub_F32 => ("sub", "f32"),
			Self::Mul_F32 => ("mul", "f32"),
			Self::Div_F32 => ("div", "f32"),
			Self::Min_F32 => ("min", "f32"),
			Self::Max_F32 => ("max", "f32"),
			Self::Copysign_F32 => ("copysign", "f32"),
			Self::Add_F64 => ("add", "f64"),
			Self::Sub_F64 => ("sub", "f64"),
			Self::Mul_F64 => ("mul", "f64"),
			Self::Div_F64 => ("div", "f64"),
			Self::Min_F64 => ("min", "f64"),
			Self::Max_F64 => ("max", "f64"),
			Self::Copysign_F64 => ("copysign", "f64"),
		}
	}
}

impl TryFrom<&Operator<'_>> for BinOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Add => Self::Add_I32,
			Operator::I32Sub => Self::Sub_I32,
			Operator::I32Mul => Self::Mul_I32,
			Operator::I32DivS => Self::DivS_I32,
			Operator::I32DivU => Self::DivU_I32,
			Operator::I32RemS => Self::RemS_I32,
			Operator::I32RemU => Self::RemU_I32,
			Operator::I32And => Self::And_I32,
			Operator::I32Or => Self::Or_I32,
			Operator::I32Xor => Self::Xor_I32,
			Operator::I32Shl => Self::Shl_I32,
			Operator::I32ShrS => Self::ShrS_I32,
			Operator::I32ShrU => Self::ShrU_I32,
			Operator::I32Rotl => Self::Rotl_I32,
			Operator::I32Rotr => Self::Rotr_I32,
			Operator::I64Add => Self::Add_I64,
			Operator::I64Sub => Self::Sub_I64,
			Operator::I64Mul => Self::Mul_I64,
			Operator::I64DivS => Self::DivS_I64,
			Operator::I64DivU => Self::DivU_I64,
			Operator::I64RemS => Self::RemS_I64,
			Operator::I64RemU => Self::RemU_I64,
			Operator::I64And => Self::And_I64,
			Operator::I64Or => Self::Or_I64,
			Operator::I64Xor => Self::Xor_I64,
			Operator::I64Shl => Self::Shl_I64,
			Operator::I64ShrS => Self::ShrS_I64,
			Operator::I64ShrU => Self::ShrU_I64,
			Operator::I64Rotl => Self::Rotl_I64,
			Operator::I64Rotr => Self::Rotr_I64,
			Operator::F32Add => Self::Add_F32,
			Operator::F32Sub => Self::Sub_F32,
			Operator::F32Mul => Self::Mul_F32,
			Operator::F32Div => Self::Div_F32,
			Operator::F32Min => Self::Min_F32,
			Operator::F32Max => Self::Max_F32,
			Operator::F32Copysign => Self::Copysign_F32,
			Operator::F64Add => Self::Add_F64,
			Operator::F64Sub => Self::Sub_F64,
			Operator::F64Mul => Self::Mul_F64,
			Operator::F64Div => Self::Div_F64,
			Operator::F64Min => Self::Min_F64,
			Operator::F64Max => Self::Max_F64,
			Operator::F64Copysign => Self::Copysign_F64,
			_ => {
				return Err(());
			}
		};

		Ok(result)
	}
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum CmpOpType {
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
	Eq_F32,
	Ne_F32,
	Lt_F32,
	Gt_F32,
	Le_F32,
	Ge_F32,
	Eq_F64,
	Ne_F64,
	Lt_F64,
	Gt_F64,
	Le_F64,
	Ge_F64,
}

impl CmpOpType {
	#[must_use]
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
			Self::Eq_F32 => ("eq", "f32"),
			Self::Ne_F32 => ("ne", "f32"),
			Self::Lt_F32 => ("lt", "f32"),
			Self::Gt_F32 => ("gt", "f32"),
			Self::Le_F32 => ("le", "f32"),
			Self::Ge_F32 => ("ge", "f32"),
			Self::Eq_F64 => ("eq", "f64"),
			Self::Ne_F64 => ("ne", "f64"),
			Self::Lt_F64 => ("lt", "f64"),
			Self::Gt_F64 => ("gt", "f64"),
			Self::Le_F64 => ("le", "f64"),
			Self::Ge_F64 => ("ge", "f64"),
		}
	}
}

impl TryFrom<&Operator<'_>> for CmpOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Eq => Self::Eq_I32,
			Operator::I32Ne => Self::Ne_I32,
			Operator::I32LtS => Self::LtS_I32,
			Operator::I32LtU => Self::LtU_I32,
			Operator::I32GtS => Self::GtS_I32,
			Operator::I32GtU => Self::GtU_I32,
			Operator::I32LeS => Self::LeS_I32,
			Operator::I32LeU => Self::LeU_I32,
			Operator::I32GeS => Self::GeS_I32,
			Operator::I32GeU => Self::GeU_I32,
			Operator::I64Eq => Self::Eq_I64,
			Operator::I64Ne => Self::Ne_I64,
			Operator::I64LtS => Self::LtS_I64,
			Operator::I64LtU => Self::LtU_I64,
			Operator::I64GtS => Self::GtS_I64,
			Operator::I64GtU => Self::GtU_I64,
			Operator::I64LeS => Self::LeS_I64,
			Operator::I64LeU => Self::LeU_I64,
			Operator::I64GeS => Self::GeS_I64,
			Operator::I64GeU => Self::GeU_I64,
			Operator::F32Eq => Self::Eq_F32,
			Operator::F32Ne => Self::Ne_F32,
			Operator::F32Lt => Self::Lt_F32,
			Operator::F32Gt => Self::Gt_F32,
			Operator::F32Le => Self::Le_F32,
			Operator::F32Ge => Self::Ge_F32,
			Operator::F64Eq => Self::Eq_F64,
			Operator::F64Ne => Self::Ne_F64,
			Operator::F64Lt => Self::Lt_F64,
			Operator::F64Gt => Self::Gt_F64,
			Operator::F64Le => Self::Le_F64,
			Operator::F64Ge => Self::Ge_F64,
			_ => {
				return Err(());
			}
		};

		Ok(result)
	}
}

pub struct Select {
	pub(crate) condition: Box<Expression>,
	pub(crate) on_true: Box<Expression>,
	pub(crate) on_false: Box<Expression>,
}

impl Select {
	#[must_use]
	pub fn condition(&self) -> &Expression {
		&self.condition
	}

	#[must_use]
	pub fn on_true(&self) -> &Expression {
		&self.on_true
	}

	#[must_use]
	pub fn on_false(&self) -> &Expression {
		&self.on_false
	}
}

pub struct GetTemporary {
	pub(crate) var: usize,
}

impl GetTemporary {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}
}

pub struct GetLocal {
	pub(crate) var: usize,
}

impl GetLocal {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}
}

pub struct GetGlobal {
	pub(crate) var: usize,
}

impl GetGlobal {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}
}

pub struct LoadAt {
	pub(crate) load_type: LoadType,
	pub(crate) memory: usize,
	pub(crate) offset: u32,
	pub(crate) pointer: Box<Expression>,
}

impl LoadAt {
	#[must_use]
	pub fn load_type(&self) -> LoadType {
		self.load_type
	}

	#[must_use]
	pub fn memory(&self) -> usize {
		self.memory
	}

	#[must_use]
	pub fn offset(&self) -> u32 {
		self.offset
	}

	#[must_use]
	pub fn pointer(&self) -> &Expression {
		&self.pointer
	}
}

pub struct MemorySize {
	pub(crate) memory: usize,
}

impl MemorySize {
	#[must_use]
	pub fn memory(&self) -> usize {
		self.memory
	}
}

#[derive(Clone, Copy)]
pub enum Value {
	I32(i32),
	I64(i64),
	F32(f32),
	F64(f64),
}

impl From<i32> for Value {
	fn from(value: i32) -> Self {
		Self::I32(value)
	}
}

impl From<i64> for Value {
	fn from(value: i64) -> Self {
		Self::I64(value)
	}
}

impl From<u32> for Value {
	fn from(value: u32) -> Self {
		Self::F32(f32::from_bits(value))
	}
}

impl From<u64> for Value {
	fn from(value: u64) -> Self {
		Self::F64(f64::from_bits(value))
	}
}

pub struct UnOp {
	pub(crate) op_type: UnOpType,
	pub(crate) rhs: Box<Expression>,
}

impl UnOp {
	#[must_use]
	pub fn op_type(&self) -> UnOpType {
		self.op_type
	}

	#[must_use]
	pub fn rhs(&self) -> &Expression {
		&self.rhs
	}
}

pub struct BinOp {
	pub(crate) op_type: BinOpType,
	pub(crate) lhs: Box<Expression>,
	pub(crate) rhs: Box<Expression>,
}

impl BinOp {
	#[must_use]
	pub fn op_type(&self) -> BinOpType {
		self.op_type
	}

	#[must_use]
	pub fn lhs(&self) -> &Expression {
		&self.lhs
	}

	#[must_use]
	pub fn rhs(&self) -> &Expression {
		&self.rhs
	}
}

pub struct CmpOp {
	pub(crate) op_type: CmpOpType,
	pub(crate) lhs: Box<Expression>,
	pub(crate) rhs: Box<Expression>,
}

impl CmpOp {
	#[must_use]
	pub fn op_type(&self) -> CmpOpType {
		self.op_type
	}

	#[must_use]
	pub fn lhs(&self) -> &Expression {
		&self.lhs
	}

	#[must_use]
	pub fn rhs(&self) -> &Expression {
		&self.rhs
	}
}

pub enum Expression {
	Select(Select),
	GetTemporary(GetTemporary),
	GetLocal(GetLocal),
	GetGlobal(GetGlobal),
	LoadAt(LoadAt),
	MemorySize(MemorySize),
	Value(Value),
	UnOp(UnOp),
	BinOp(BinOp),
	CmpOp(CmpOp),
}

pub struct Align {
	pub(crate) new: usize,
	pub(crate) old: usize,
	pub(crate) length: usize,
}

impl Align {
	#[must_use]
	pub fn is_aligned(&self) -> bool {
		self.length == 0 || self.new == self.old
	}

	#[must_use]
	pub fn new_range(&self) -> Range<usize> {
		self.new..self.new + self.length
	}

	#[must_use]
	pub fn old_range(&self) -> Range<usize> {
		self.old..self.old + self.length
	}
}

pub struct Br {
	pub(crate) target: usize,
	pub(crate) align: Align,
}

impl Br {
	#[must_use]
	pub fn target(&self) -> usize {
		self.target
	}

	#[must_use]
	pub fn align(&self) -> &Align {
		&self.align
	}
}

pub struct BrTable {
	pub(crate) condition: Box<Expression>,
	pub(crate) data: Vec<Br>,
	pub(crate) default: Br,
}

impl BrTable {
	#[must_use]
	pub fn condition(&self) -> &Expression {
		&self.condition
	}

	#[must_use]
	pub fn data(&self) -> &[Br] {
		&self.data
	}

	#[must_use]
	pub fn default(&self) -> &Br {
		&self.default
	}
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum LabelType {
	Forward,
	Backward,
}

pub enum Terminator {
	Unreachable,
	Br(Br),
	BrTable(BrTable),
}

#[derive(Default)]
pub struct Block {
	pub(crate) label_type: Option<LabelType>,
	pub(crate) code: Vec<Statement>,
	pub(crate) last: Option<Terminator>,
}

impl Block {
	#[must_use]
	pub fn label_type(&self) -> Option<LabelType> {
		self.label_type
	}

	#[must_use]
	pub fn code(&self) -> &[Statement] {
		&self.code
	}

	#[must_use]
	pub fn last(&self) -> Option<&Terminator> {
		self.last.as_ref()
	}
}

pub struct BrIf {
	pub(crate) condition: Box<Expression>,
	pub(crate) target: Br,
}

impl BrIf {
	#[must_use]
	pub fn condition(&self) -> &Expression {
		&self.condition
	}

	#[must_use]
	pub fn target(&self) -> &Br {
		&self.target
	}
}

pub struct If {
	pub(crate) condition: Box<Expression>,
	pub(crate) on_true: Block,
	pub(crate) on_false: Option<Block>,
}

impl If {
	#[must_use]
	pub fn condition(&self) -> &Expression {
		&self.condition
	}

	#[must_use]
	pub fn on_true(&self) -> &Block {
		&self.on_true
	}

	#[must_use]
	pub fn on_false(&self) -> Option<&Block> {
		self.on_false.as_ref()
	}
}

pub struct Call {
	pub(crate) function: usize,
	pub(crate) result: Range<usize>,
	pub(crate) param_list: Vec<Expression>,
}

impl Call {
	#[must_use]
	pub fn function(&self) -> usize {
		self.function
	}

	#[must_use]
	pub fn result(&self) -> Range<usize> {
		self.result.clone()
	}

	#[must_use]
	pub fn param_list(&self) -> &[Expression] {
		&self.param_list
	}
}

pub struct CallIndirect {
	pub(crate) table: usize,
	pub(crate) index: Box<Expression>,
	pub(crate) result: Range<usize>,
	pub(crate) param_list: Vec<Expression>,
}

impl CallIndirect {
	#[must_use]
	pub fn table(&self) -> usize {
		self.table
	}

	#[must_use]
	pub fn index(&self) -> &Expression {
		&self.index
	}

	#[must_use]
	pub fn result(&self) -> Range<usize> {
		self.result.clone()
	}

	#[must_use]
	pub fn param_list(&self) -> &[Expression] {
		&self.param_list
	}
}

pub struct SetTemporary {
	pub(crate) var: usize,
	pub(crate) value: Box<Expression>,
}

impl SetTemporary {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}

	#[must_use]
	pub fn value(&self) -> &Expression {
		&self.value
	}
}

pub struct SetLocal {
	pub(crate) var: usize,
	pub(crate) value: Box<Expression>,
}

impl SetLocal {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}

	#[must_use]
	pub fn value(&self) -> &Expression {
		&self.value
	}
}

pub struct SetGlobal {
	pub(crate) var: usize,
	pub(crate) value: Box<Expression>,
}

impl SetGlobal {
	#[must_use]
	pub fn var(&self) -> usize {
		self.var
	}

	#[must_use]
	pub fn value(&self) -> &Expression {
		&self.value
	}
}

pub struct StoreAt {
	pub(crate) store_type: StoreType,
	pub(crate) memory: usize,
	pub(crate) offset: u32,
	pub(crate) pointer: Box<Expression>,
	pub(crate) value: Box<Expression>,
}

impl StoreAt {
	#[must_use]
	pub fn store_type(&self) -> StoreType {
		self.store_type
	}

	#[must_use]
	pub fn memory(&self) -> usize {
		self.memory
	}

	#[must_use]
	pub fn offset(&self) -> u32 {
		self.offset
	}

	#[must_use]
	pub fn pointer(&self) -> &Expression {
		&self.pointer
	}

	#[must_use]
	pub fn value(&self) -> &Expression {
		&self.value
	}
}

pub struct MemoryGrow {
	pub(crate) memory: usize,
	pub(crate) result: usize,
	pub(crate) size: Box<Expression>,
}

impl MemoryGrow {
	#[must_use]
	pub fn memory(&self) -> usize {
		self.memory
	}

	#[must_use]
	pub fn result(&self) -> usize {
		self.result
	}

	#[must_use]
	pub fn size(&self) -> &Expression {
		&self.size
	}
}

pub struct MemoryCopy {
	pub(crate) dst: u32,
	pub(crate) src: u32,
	pub(crate) size: Box<Expression>,
}

impl MemoryCopy {
	#[must_use]
	pub fn dst(&self) -> u32 {
		self.dst
	}
	#[must_use]
	pub fn src(&self) -> u32 {
		self.src
	}
	#[must_use]
	pub fn size(&self) -> &Expression {
		&self.size
	}
}

pub struct MemoryFill {
	pub(crate) mem: u32,
	pub(crate) value: Box<Expression>,
	pub(crate) n: Box<Expression>,
}

impl MemoryFill {
	#[must_use]
	pub fn mem(&self) -> u32 {
		self.mem
	}
	#[must_use]
	pub fn value(&self) -> &Expression {
		&self.value
	}
	#[must_use]
	pub fn n(&self) -> &Expression {
		&self.n
	}
}

pub enum Statement {
	Block(Block),
	BrIf(BrIf),
	If(If),
	Call(Call),
	CallIndirect(CallIndirect),
	SetTemporary(SetTemporary),
	SetLocal(SetLocal),
	SetGlobal(SetGlobal),
	StoreAt(StoreAt),
	MemoryGrow(MemoryGrow),
	MemoryCopy(MemoryCopy),
	MemoryFill(MemoryFill)
}

pub struct FuncData {
	pub(crate) local_data: Vec<(u32, ValType)>,
	pub(crate) num_result: usize,
	pub(crate) num_param: usize,
	pub(crate) num_stack: usize,
	pub(crate) code: Block,
}

impl FuncData {
	#[must_use]
	pub fn local_data(&self) -> &[(u32, ValType)] {
		&self.local_data
	}

	#[must_use]
	pub fn num_result(&self) -> usize {
		self.num_result
	}

	#[must_use]
	pub fn num_param(&self) -> usize {
		self.num_param
	}

	#[must_use]
	pub fn num_stack(&self) -> usize {
		self.num_stack
	}

	#[must_use]
	pub fn code(&self) -> &Block {
		&self.code
	}
}
