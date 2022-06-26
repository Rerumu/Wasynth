use wasm_ast::node::{BinOpType, CmpOpType};

pub trait AsSymbol {
	fn as_symbol(&self) -> Option<&'static str>;
}

impl AsSymbol for BinOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Add_F32 | Self::Add_F64 => "+",
			Self::Sub_F32 | Self::Sub_F64 => "-",
			Self::Mul_F32 | Self::Mul_F64 => "*",
			Self::Div_F32 | Self::Div_F64 => "/",
			Self::RemU_I32 => "%",
			_ => return None,
		};

		Some(result)
	}
}

impl AsSymbol for CmpOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Eq_I32 | Self::Eq_F32 | Self::Eq_F64 => "==",
			Self::Ne_I32 | Self::Ne_F32 | Self::Ne_F64 => "~=",
			Self::LtU_I32 | Self::Lt_F32 | Self::Lt_F64 => "<",
			Self::GtU_I32 | Self::Gt_F32 | Self::Gt_F64 => ">",
			Self::LeU_I32 | Self::Le_F32 | Self::Le_F64 => "<=",
			Self::GeU_I32 | Self::Ge_F32 | Self::Ge_F64 => ">=",
			_ => return None,
		};

		Some(result)
	}
}
