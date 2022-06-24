use wasm_ast::node::{BinOpType, CmpOpType};

pub trait AsSymbol {
	fn as_symbol(&self) -> Option<&'static str>;
}

impl AsSymbol for BinOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Add_I64 | Self::Add_FN => "+",
			Self::Sub_I64 | Self::Sub_FN => "-",
			Self::Mul_I64 | Self::Mul_FN => "*",
			Self::DivS_I64 | Self::Div_FN => "/",
			Self::RemS_I64 => "%",
			_ => return None,
		};

		Some(result)
	}
}

impl AsSymbol for CmpOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Eq_I32 | Self::Eq_I64 | Self::Eq_FN => "==",
			Self::Ne_I32 | Self::Ne_I64 | Self::Ne_FN => "~=",
			Self::LtS_I32 | Self::LtS_I64 | Self::Lt_FN => "<",
			Self::GtS_I32 | Self::GtS_I64 | Self::Gt_FN => ">",
			Self::LeS_I32 | Self::LeS_I64 | Self::Le_FN => "<=",
			Self::GeS_I32 | Self::GeS_I64 | Self::Ge_FN => ">=",
			_ => return None,
		};

		Some(result)
	}
}
