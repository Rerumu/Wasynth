#![allow(non_camel_case_types)]

use wasmparser::{MemoryImmediate, Operator};

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
	pub fn try_extract(inst: &Operator) -> Result<(Self, MemoryImmediate), ()> {
		let result = match *inst {
			Operator::I32Load { memarg } => (Self::I32, memarg),
			Operator::I64Load { memarg } => (Self::I64, memarg),
			Operator::F32Load { memarg } => (Self::F32, memarg),
			Operator::F64Load { memarg } => (Self::F64, memarg),
			Operator::I32Load8S { memarg } => (Self::I32_I8, memarg),
			Operator::I32Load8U { memarg } => (Self::I32_U8, memarg),
			Operator::I32Load16S { memarg } => (Self::I32_I16, memarg),
			Operator::I32Load16U { memarg } => (Self::I32_U16, memarg),
			Operator::I64Load8S { memarg } => (Self::I64_I8, memarg),
			Operator::I64Load8U { memarg } => (Self::I64_U8, memarg),
			Operator::I64Load16S { memarg } => (Self::I64_I16, memarg),
			Operator::I64Load16U { memarg } => (Self::I64_U16, memarg),
			Operator::I64Load32S { memarg } => (Self::I64_I32, memarg),
			Operator::I64Load32U { memarg } => (Self::I64_U32, memarg),
			_ => return Err(()),
		};

		Ok(result)
	}
}

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
	pub fn try_extract(inst: &Operator) -> Result<(Self, MemoryImmediate), ()> {
		let result = match *inst {
			Operator::I32Store { memarg } => (Self::I32, memarg),
			Operator::I64Store { memarg } => (Self::I64, memarg),
			Operator::F32Store { memarg } => (Self::F32, memarg),
			Operator::F64Store { memarg } => (Self::F64, memarg),
			Operator::I32Store8 { memarg } => (Self::I32_N8, memarg),
			Operator::I32Store16 { memarg } => (Self::I32_N16, memarg),
			Operator::I64Store8 { memarg } => (Self::I64_N8, memarg),
			Operator::I64Store16 { memarg } => (Self::I64_N16, memarg),
			Operator::I64Store32 { memarg } => (Self::I64_N32, memarg),
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[derive(Clone, Copy)]
pub enum IUnOpType {
	Clz,
	Ctz,
	Popcnt,
}

#[derive(Clone, Copy)]
pub enum FUnOpType {
	Abs,
	Ceil,
	Floor,
	Nearest,
	Neg,
	Sqrt,
	Truncate,
}

#[derive(Clone, Copy)]
pub enum UnOpType {
	I32(IUnOpType),
	I64(IUnOpType),
	F32(FUnOpType),
	F64(FUnOpType),
}

impl TryFrom<&Operator<'_>> for UnOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Clz => Self::I32(IUnOpType::Clz),
			Operator::I64Clz => Self::I64(IUnOpType::Clz),
			Operator::I32Ctz => Self::I32(IUnOpType::Ctz),
			Operator::I64Ctz => Self::I64(IUnOpType::Ctz),
			Operator::I32Popcnt => Self::I32(IUnOpType::Popcnt),
			Operator::I64Popcnt => Self::I64(IUnOpType::Popcnt),
			Operator::F32Abs => Self::F32(FUnOpType::Abs),
			Operator::F64Abs => Self::F64(FUnOpType::Abs),
			Operator::F32Ceil => Self::F32(FUnOpType::Ceil),
			Operator::F64Ceil => Self::F64(FUnOpType::Ceil),
			Operator::F32Floor => Self::F32(FUnOpType::Floor),
			Operator::F64Floor => Self::F64(FUnOpType::Floor),
			Operator::F32Nearest => Self::F32(FUnOpType::Nearest),
			Operator::F64Nearest => Self::F64(FUnOpType::Nearest),
			Operator::F32Neg => Self::F32(FUnOpType::Neg),
			Operator::F64Neg => Self::F64(FUnOpType::Neg),
			Operator::F32Sqrt => Self::F32(FUnOpType::Sqrt),
			Operator::F64Sqrt => Self::F64(FUnOpType::Sqrt),
			Operator::F32Trunc => Self::F32(FUnOpType::Truncate),
			Operator::F64Trunc => Self::F64(FUnOpType::Truncate),
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[derive(Clone, Copy)]
pub enum IBinOpType {
	Add,
	And,
	DivS,
	DivU,
	Mul,
	Or,
	RemS,
	RemU,
	Rotl,
	Rotr,
	Shl,
	ShrS,
	ShrU,
	Sub,
	Xor,
}

#[derive(Clone, Copy)]
pub enum FBinOpType {
	Add,
	Copysign,
	Div,
	Max,
	Min,
	Mul,
	Sub,
}

#[derive(Clone, Copy)]
pub enum BinOpType {
	I32(IBinOpType),
	I64(IBinOpType),
	F32(FBinOpType),
	F64(FBinOpType),
}

impl TryFrom<&Operator<'_>> for BinOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Add => Self::I32(IBinOpType::Add),
			Operator::I64Add => Self::I64(IBinOpType::Add),
			Operator::I32And => Self::I32(IBinOpType::And),
			Operator::I64And => Self::I64(IBinOpType::And),
			Operator::I32DivS => Self::I32(IBinOpType::DivS),
			Operator::I64DivS => Self::I64(IBinOpType::DivS),
			Operator::I32DivU => Self::I32(IBinOpType::DivU),
			Operator::I64DivU => Self::I64(IBinOpType::DivU),
			Operator::I32Mul => Self::I32(IBinOpType::Mul),
			Operator::I64Mul => Self::I64(IBinOpType::Mul),
			Operator::I32Or => Self::I32(IBinOpType::Or),
			Operator::I64Or => Self::I64(IBinOpType::Or),
			Operator::I32RemS => Self::I32(IBinOpType::RemS),
			Operator::I64RemS => Self::I64(IBinOpType::RemS),
			Operator::I32RemU => Self::I32(IBinOpType::RemU),
			Operator::I64RemU => Self::I64(IBinOpType::RemU),
			Operator::I32Rotl => Self::I32(IBinOpType::Rotl),
			Operator::I64Rotl => Self::I64(IBinOpType::Rotl),
			Operator::I32Rotr => Self::I32(IBinOpType::Rotr),
			Operator::I64Rotr => Self::I64(IBinOpType::Rotr),
			Operator::I32Shl => Self::I32(IBinOpType::Shl),
			Operator::I64Shl => Self::I64(IBinOpType::Shl),
			Operator::I32ShrS => Self::I32(IBinOpType::ShrS),
			Operator::I64ShrS => Self::I64(IBinOpType::ShrS),
			Operator::I32ShrU => Self::I32(IBinOpType::ShrU),
			Operator::I64ShrU => Self::I64(IBinOpType::ShrU),
			Operator::I32Sub => Self::I32(IBinOpType::Sub),
			Operator::I64Sub => Self::I64(IBinOpType::Sub),
			Operator::I32Xor => Self::I32(IBinOpType::Xor),
			Operator::I64Xor => Self::I64(IBinOpType::Xor),
			Operator::F32Add => Self::F32(FBinOpType::Add),
			Operator::F64Add => Self::F64(FBinOpType::Add),
			Operator::F32Copysign => Self::F32(FBinOpType::Copysign),
			Operator::F64Copysign => Self::F64(FBinOpType::Copysign),
			Operator::F32Div => Self::F32(FBinOpType::Div),
			Operator::F64Div => Self::F64(FBinOpType::Div),
			Operator::F32Max => Self::F32(FBinOpType::Max),
			Operator::F64Max => Self::F64(FBinOpType::Max),
			Operator::F32Min => Self::F32(FBinOpType::Min),
			Operator::F64Min => Self::F64(FBinOpType::Min),
			Operator::F32Mul => Self::F32(FBinOpType::Mul),
			Operator::F64Mul => Self::F64(FBinOpType::Mul),
			Operator::F32Sub => Self::F32(FBinOpType::Sub),
			Operator::F64Sub => Self::F64(FBinOpType::Sub),
			_ => return Err(()),
		};

		Ok(result)
	}
}

#[derive(Clone, Copy)]
pub enum ICmpOpType {
	Eq,
	GeS,
	GeU,
	GtS,
	GtU,
	LeS,
	LeU,
	LtS,
	LtU,
	Ne,
}

#[derive(Clone, Copy)]
pub enum FCmpOpType {
	Eq,
	Ge,
	Gt,
	Le,
	Lt,
	Ne,
}

#[derive(Clone, Copy)]
pub enum CmpOpType {
	I32(ICmpOpType),
	I64(ICmpOpType),
	F32(FCmpOpType),
	F64(FCmpOpType),
}

impl TryFrom<&Operator<'_>> for CmpOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
			Operator::I32Eq => Self::I32(ICmpOpType::Eq),
			Operator::I64Eq => Self::I64(ICmpOpType::Eq),
			Operator::I32GeS => Self::I32(ICmpOpType::GeS),
			Operator::I64GeS => Self::I64(ICmpOpType::GeS),
			Operator::I32GeU => Self::I32(ICmpOpType::GeU),
			Operator::I64GeU => Self::I64(ICmpOpType::GeU),
			Operator::I32GtS => Self::I32(ICmpOpType::GtS),
			Operator::I64GtS => Self::I64(ICmpOpType::GtS),
			Operator::I32GtU => Self::I32(ICmpOpType::GtU),
			Operator::I64GtU => Self::I64(ICmpOpType::GtU),
			Operator::I32LeS => Self::I32(ICmpOpType::LeS),
			Operator::I64LeS => Self::I64(ICmpOpType::LeS),
			Operator::I32LeU => Self::I32(ICmpOpType::LeU),
			Operator::I64LeU => Self::I64(ICmpOpType::LeU),
			Operator::I32LtS => Self::I32(ICmpOpType::LtS),
			Operator::I64LtS => Self::I64(ICmpOpType::LtS),
			Operator::I32LtU => Self::I32(ICmpOpType::LtU),
			Operator::I64LtU => Self::I64(ICmpOpType::LtU),
			Operator::I32Ne => Self::I32(ICmpOpType::Ne),
			Operator::I64Ne => Self::I64(ICmpOpType::Ne),
			Operator::F32Eq => Self::F32(FCmpOpType::Eq),
			Operator::F64Eq => Self::F64(FCmpOpType::Eq),
			Operator::F32Ge => Self::F32(FCmpOpType::Ge),
			Operator::F64Ge => Self::F64(FCmpOpType::Ge),
			Operator::F32Gt => Self::F32(FCmpOpType::Gt),
			Operator::F64Gt => Self::F64(FCmpOpType::Gt),
			Operator::F32Le => Self::F32(FCmpOpType::Le),
			Operator::F64Le => Self::F64(FCmpOpType::Le),
			Operator::F32Lt => Self::F32(FCmpOpType::Lt),
			Operator::F64Lt => Self::F64(FCmpOpType::Lt),
			Operator::F32Ne => Self::F32(FCmpOpType::Ne),
			Operator::F64Ne => Self::F64(FCmpOpType::Ne),
			_ => return Err(()),
		};

		Ok(result)
	}
}

// Order of mnemonics is:
// operation_result_parameter
#[derive(Clone, Copy)]
pub enum CastOpType {
	Convert_F32_I32,
	Convert_F32_I64,
	Convert_F32_U32,
	Convert_F32_U64,
	Convert_F64_I32,
	Convert_F64_I64,
	Convert_F64_U32,
	Convert_F64_U64,
	Demote_F32_F64,
	Extend_I32_N16,
	Extend_I32_N8,
	Extend_I64_I32,
	Extend_I64_N16,
	Extend_I64_N32,
	Extend_I64_N8,
	Extend_I64_U32,
	Promote_F64_F32,
	Reinterpret_F32_I32,
	Reinterpret_F64_I64,
	Reinterpret_I32_F32,
	Reinterpret_I64_F64,
	Saturate_I32_F32,
	Saturate_I32_F64,
	Saturate_I64_F32,
	Saturate_I64_F64,
	Saturate_U32_F32,
	Saturate_U32_F64,
	Saturate_U64_F32,
	Saturate_U64_F64,
	Truncate_I32_F32,
	Truncate_I32_F64,
	Truncate_I64_F32,
	Truncate_I64_F64,
	Truncate_U32_F32,
	Truncate_U32_F64,
	Truncate_U64_F32,
	Truncate_U64_F64,
	Wrap_I32_I64,
}

impl TryFrom<&Operator<'_>> for CastOpType {
	type Error = ();

	fn try_from(inst: &Operator) -> Result<Self, Self::Error> {
		let result = match inst {
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
