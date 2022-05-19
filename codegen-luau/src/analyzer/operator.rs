use wasm_ast::node::{BinOpType, CmpOpType};

pub fn bin_symbol_of(op: BinOpType) -> Option<&'static str> {
	let result = match op {
		BinOpType::Add_FN => "+",
		BinOpType::Sub_FN => "-",
		BinOpType::Mul_FN => "*",
		BinOpType::Div_FN => "/",
		BinOpType::RemS_I32 | BinOpType::RemU_I32 => "%",
		_ => return None,
	};

	Some(result)
}

pub fn cmp_symbol_of(op: CmpOpType) -> Option<&'static str> {
	let result = match op {
		CmpOpType::Eq_I32 | CmpOpType::Eq_FN => "==",
		CmpOpType::Ne_I32 | CmpOpType::Ne_FN => "~=",
		CmpOpType::LtU_I32 | CmpOpType::Lt_FN => "<",
		CmpOpType::GtU_I32 | CmpOpType::Gt_FN => ">",
		CmpOpType::LeU_I32 | CmpOpType::Le_FN => "<=",
		CmpOpType::GeU_I32 | CmpOpType::Ge_FN => ">=",
		_ => return None,
	};

	Some(result)
}
