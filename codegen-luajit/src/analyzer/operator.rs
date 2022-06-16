use wasm_ast::node::{BinOpType, CmpOpType};

pub fn bin_symbol_of(op: BinOpType) -> Option<&'static str> {
	let result = match op {
		BinOpType::Add_I64 | BinOpType::Add_FN => "+",
		BinOpType::Sub_I64 | BinOpType::Sub_FN => "-",
		BinOpType::Mul_I64 | BinOpType::Mul_FN => "*",
		BinOpType::DivS_I64 | BinOpType::Div_FN => "/",
		BinOpType::RemS_I64 => "%",
		_ => return None,
	};

	Some(result)
}

pub fn cmp_symbol_of(op: CmpOpType) -> Option<&'static str> {
	let result = match op {
		CmpOpType::Eq_I32 | CmpOpType::Eq_I64 | CmpOpType::Eq_FN => "==",
		CmpOpType::Ne_I32 | CmpOpType::Ne_I64 | CmpOpType::Ne_FN => "~=",
		CmpOpType::LtS_I32 | CmpOpType::LtS_I64 | CmpOpType::Lt_FN => "<",
		CmpOpType::GtS_I32 | CmpOpType::GtS_I64 | CmpOpType::Gt_FN => ">",
		CmpOpType::LeS_I32 | CmpOpType::LeS_I64 | CmpOpType::Le_FN => "<=",
		CmpOpType::GeS_I32 | CmpOpType::GeS_I64 | CmpOpType::Ge_FN => ">=",
		_ => return None,
	};

	Some(result)
}
