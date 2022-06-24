use wasm_ast::node::{BinOpType, CmpOpType};

pub trait AsSymbol {
	fn as_symbol(&self) -> Option<&'static str>;
}

impl AsSymbol for BinOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Add_FN => "+",
			Self::Sub_FN => "-",
			Self::Mul_FN => "*",
			Self::Div_FN => "/",
			Self::RemS_I32 | Self::RemU_I32 => "%",
			_ => return None,
		};

		Some(result)
	}
}

impl AsSymbol for CmpOpType {
	fn as_symbol(&self) -> Option<&'static str> {
		let result = match self {
			Self::Eq_I32 | Self::Eq_FN => "==",
			Self::Ne_I32 | Self::Ne_FN => "~=",
			Self::LtU_I32 | Self::Lt_FN => "<",
			Self::GtU_I32 | Self::Gt_FN => ">",
			Self::LeU_I32 | Self::Le_FN => "<=",
			Self::GeU_I32 | Self::Ge_FN => ">=",
			_ => return None,
		};

		Some(result)
	}
}
