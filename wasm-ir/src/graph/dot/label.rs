use crate::graph::{
	discriminant::{
		BinOpType, CastOpType, CmpOpType, FBinOpType, FCmpOpType, FUnOpType, IBinOpType,
		ICmpOpType, IUnOpType, LoadType, StoreType, UnOpType,
	},
	node::{Number, Ordering, Simple},
};

fn label_of_ordering(order: &Ordering) -> String {
	match order {
		Ordering::Memory => "global order",
		Ordering::Global => "memory order",
	}
	.into()
}

fn label_of_number(number: &Number) -> String {
	match number {
		Number::I32(v) => format!("i32 {v}"),
		Number::I64(v) => format!("i64 {v}"),
		Number::F32(v) => format!("f32 {v}"),
		Number::F64(v) => format!("f64 {v}"),
	}
}

fn label_of_load_type(load: LoadType) -> String {
	let postfix = match load {
		LoadType::I32 => "i32",
		LoadType::I64 => "i64",
		LoadType::F32 => "f32",
		LoadType::F64 => "f64",
		LoadType::I32_I8 => "i32_i8",
		LoadType::I32_U8 => "i32_u8",
		LoadType::I32_I16 => "i32_i16",
		LoadType::I32_U16 => "i32_u16",
		LoadType::I64_I8 => "i64_i8",
		LoadType::I64_U8 => "i64_u8",
		LoadType::I64_I16 => "i64_i16",
		LoadType::I64_U16 => "i64_u16",
		LoadType::I64_I32 => "i64_i32",
		LoadType::I64_U32 => "i64_u32",
	};

	format!("load {postfix}")
}

fn label_of_store_type(store: StoreType) -> String {
	let postfix = match store {
		StoreType::I32 => "i32",
		StoreType::I64 => "i64",
		StoreType::F32 => "f32",
		StoreType::F64 => "f64",
		StoreType::I32_N8 => "i32_n8",
		StoreType::I32_N16 => "i32_n16",
		StoreType::I64_N8 => "i64_n8",
		StoreType::I64_N16 => "i64_n16",
		StoreType::I64_N32 => "i64_n32",
	};

	format!("store {postfix}")
}

fn label_of_i_unop(op: IUnOpType) -> String {
	let name = match op {
		IUnOpType::Clz => "clz",
		IUnOpType::Ctz => "ctz",
		IUnOpType::Popcnt => "popcnt",
	};

	name.into()
}

fn label_of_f_unop(op: FUnOpType) -> String {
	let name = match op {
		FUnOpType::Abs => "abs",
		FUnOpType::Ceil => "ceil",
		FUnOpType::Floor => "floor",
		FUnOpType::Nearest => "nearest",
		FUnOpType::Neg => "neg",
		FUnOpType::Sqrt => "sqrt",
		FUnOpType::Truncate => "truncate",
	};

	name.into()
}

fn label_of_un_op(op: UnOpType) -> String {
	match op {
		UnOpType::I32(v) | UnOpType::I64(v) => label_of_i_unop(v),
		UnOpType::F32(v) | UnOpType::F64(v) => label_of_f_unop(v),
	}
}

fn label_of_i_bin_op(op: IBinOpType) -> String {
	let name = match op {
		IBinOpType::Add => "add",
		IBinOpType::And => "and",
		IBinOpType::DivS => "div_s",
		IBinOpType::DivU => "div_u",
		IBinOpType::Mul => "mul",
		IBinOpType::Or => "or",
		IBinOpType::RemS => "rem_s",
		IBinOpType::RemU => "rem_u",
		IBinOpType::Rotl => "rotl",
		IBinOpType::Rotr => "rotr",
		IBinOpType::Shl => "shl",
		IBinOpType::ShrS => "shr_s",
		IBinOpType::ShrU => "shr_u",
		IBinOpType::Sub => "sub",
		IBinOpType::Xor => "xor",
	};

	name.into()
}

fn label_of_f_bin_op(op: FBinOpType) -> String {
	let name = match op {
		FBinOpType::Add => "add",
		FBinOpType::Div => "div",
		FBinOpType::Max => "max",
		FBinOpType::Min => "min",
		FBinOpType::Mul => "mul",
		FBinOpType::Copysign => "copysign",
		FBinOpType::Sub => "sub",
	};

	name.into()
}

fn label_of_bin_op(op: BinOpType) -> String {
	match op {
		BinOpType::I32(v) | BinOpType::I64(v) => label_of_i_bin_op(v),
		BinOpType::F32(v) | BinOpType::F64(v) => label_of_f_bin_op(v),
	}
}

fn label_of_i_cmp_op(op: ICmpOpType) -> String {
	let name = match op {
		ICmpOpType::Eq => "eq",
		ICmpOpType::GeS => "ge_s",
		ICmpOpType::GeU => "ge_u",
		ICmpOpType::GtS => "gt_s",
		ICmpOpType::GtU => "gt_u",
		ICmpOpType::LeS => "le_s",
		ICmpOpType::LeU => "le_u",
		ICmpOpType::LtS => "lt_s",
		ICmpOpType::LtU => "lt_u",
		ICmpOpType::Ne => "ne",
	};

	name.into()
}

fn label_of_f_cmp_op(op: FCmpOpType) -> String {
	let name = match op {
		FCmpOpType::Eq => "eq",
		FCmpOpType::Ge => "ge",
		FCmpOpType::Gt => "gt",
		FCmpOpType::Le => "le",
		FCmpOpType::Lt => "lt",
		FCmpOpType::Ne => "ne",
	};

	name.into()
}

fn label_of_cmp_op(op: CmpOpType) -> String {
	match op {
		CmpOpType::I32(v) | CmpOpType::I64(v) => label_of_i_cmp_op(v),
		CmpOpType::F32(v) | CmpOpType::F64(v) => label_of_f_cmp_op(v),
	}
}

fn label_of_cast_op(op: CastOpType) -> String {
	let name = match op {
		CastOpType::Convert_F32_I32 => "convert_f32_i32",
		CastOpType::Convert_F32_I64 => "convert_f32_i64",
		CastOpType::Convert_F32_U32 => "convert_f32_u32",
		CastOpType::Convert_F32_U64 => "convert_f32_u64",
		CastOpType::Convert_F64_I32 => "convert_f64_i32",
		CastOpType::Convert_F64_I64 => "convert_f64_i64",
		CastOpType::Convert_F64_U32 => "convert_f64_u32",
		CastOpType::Convert_F64_U64 => "convert_f64_u64",
		CastOpType::Demote_F32_F64 => "demote_f32_f64",
		CastOpType::Extend_I32_N16 => "extend_i32_n16",
		CastOpType::Extend_I32_N8 => "extend_i32_n8",
		CastOpType::Extend_I64_I32 => "extend_i64_i32",
		CastOpType::Extend_I64_N16 => "extend_i64_n16",
		CastOpType::Extend_I64_N32 => "extend_i64_n32",
		CastOpType::Extend_I64_N8 => "extend_i64_n8",
		CastOpType::Extend_I64_U32 => "extend_i64_u32",
		CastOpType::Promote_F64_F32 => "promote_f64_f32",
		CastOpType::Reinterpret_F32_I32 => "reinterpret_f32_i32",
		CastOpType::Reinterpret_F64_I64 => "reinterpret_f64_i64",
		CastOpType::Reinterpret_I32_F32 => "reinterpret_i32_f32",
		CastOpType::Reinterpret_I64_F64 => "reinterpret_i64_f64",
		CastOpType::Saturate_I32_F32 => "saturate_i32_f32",
		CastOpType::Saturate_I32_F64 => "saturate_i32_f64",
		CastOpType::Saturate_I64_F32 => "saturate_i64_f32",
		CastOpType::Saturate_I64_F64 => "saturate_i64_f64",
		CastOpType::Saturate_U32_F32 => "saturate_u32_f32",
		CastOpType::Saturate_U32_F64 => "saturate_u32_f64",
		CastOpType::Saturate_U64_F32 => "saturate_u64_f32",
		CastOpType::Saturate_U64_F64 => "saturate_u64_f64",
		CastOpType::Truncate_I32_F32 => "truncate_i32_f32",
		CastOpType::Truncate_I32_F64 => "truncate_i32_f64",
		CastOpType::Truncate_I64_F32 => "truncate_i64_f32",
		CastOpType::Truncate_I64_F64 => "truncate_i64_f64",
		CastOpType::Truncate_U32_F32 => "truncate_u32_f32",
		CastOpType::Truncate_U32_F64 => "truncate_u32_f64",
		CastOpType::Truncate_U64_F32 => "truncate_u64_f32",
		CastOpType::Truncate_U64_F64 => "truncate_u64_f64",
		CastOpType::Wrap_I32_I64 => "wrap_i32_i64",
	};

	name.into()
}

pub fn label_of_simple(simple: &Simple) -> String {
	match simple {
		Simple::Undefined(_) => "undefined".into(),
		Simple::Ordering(v) => label_of_ordering(v),
		Simple::Unreachable(_) => "unreachable".into(),
		Simple::Argument(_) => "argument".into(),
		Simple::Number(v) => label_of_number(v),
		Simple::GetFunction(v) => format!("get function {}", v.function),
		Simple::GetTableElement(v) => format!("get table element {}", v.table),
		Simple::Call(_) => "call".to_string(),
		Simple::GlobalGet(v) => format!("global get {}", v.global),
		Simple::GlobalSet(v) => format!("global set {}", v.global),
		Simple::Load(v) => label_of_load_type(v.load_type),
		Simple::Store(v) => label_of_store_type(v.store_type),
		Simple::MemorySize(v) => format!("memory size {}", v.memory),
		Simple::MemoryGrow(v) => format!("memory grow {}", v.memory),
		Simple::UnOp(v) => label_of_un_op(v.op_type),
		Simple::BinOp(v) => label_of_bin_op(v.op_type),
		Simple::CmpOp(v) => label_of_cmp_op(v.op_type),
		Simple::CastOp(v) => label_of_cast_op(v.op_type),
	}
}
