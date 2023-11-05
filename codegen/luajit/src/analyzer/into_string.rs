use wasm_ast::node::{BinOpType, CmpOpType, LoadType, StoreType, UnOpType};

pub trait IntoName {
	#[must_use]
	fn into_name(self) -> &'static str;
}

impl IntoName for LoadType {
	fn into_name(self) -> &'static str {
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

impl IntoName for StoreType {
	fn into_name(self) -> &'static str {
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

pub trait IntoNameTuple {
	#[must_use]
	fn into_name_tuple(self) -> (&'static str, &'static str);
}

impl IntoNameTuple for UnOpType {
	fn into_name_tuple(self) -> (&'static str, &'static str) {
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

impl IntoNameTuple for BinOpType {
	fn into_name_tuple(self) -> (&'static str, &'static str) {
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

impl IntoNameTuple for CmpOpType {
	fn into_name_tuple(self) -> (&'static str, &'static str) {
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

pub trait TryIntoSymbol {
	#[must_use]
	fn try_into_symbol(self) -> Option<&'static str>;
}

impl TryIntoSymbol for BinOpType {
	fn try_into_symbol(self) -> Option<&'static str> {
		let result = match self {
			Self::Add_I64 | Self::Add_F32 | Self::Add_F64 => "+",
			Self::Sub_I64 | Self::Sub_F32 | Self::Sub_F64 => "-",
			Self::Mul_I64 | Self::Mul_F32 | Self::Mul_F64 => "*",
			Self::DivS_I64 | Self::Div_F32 | Self::Div_F64 => "/",
			Self::RemS_I64 => "%",
			_ => return None,
		};

		Some(result)
	}
}

impl TryIntoSymbol for CmpOpType {
	fn try_into_symbol(self) -> Option<&'static str> {
		let result = match self {
			Self::Eq_I32 | Self::Eq_I64 | Self::Eq_F32 | Self::Eq_F64 => "==",
			Self::Ne_I32 | Self::Ne_I64 | Self::Ne_F32 | Self::Ne_F64 => "~=",
			Self::LtS_I32 | Self::LtS_I64 | Self::Lt_F32 | Self::Lt_F64 => "<",
			Self::GtS_I32 | Self::GtS_I64 | Self::Gt_F32 | Self::Gt_F64 => ">",
			Self::LeS_I32 | Self::LeS_I64 | Self::Le_F32 | Self::Le_F64 => "<=",
			Self::GeS_I32 | Self::GeS_I64 | Self::Ge_F32 | Self::Ge_F64 => ">=",
			_ => return None,
		};

		Some(result)
	}
}
