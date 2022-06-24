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
	Abs_FN,
	Neg_FN,
	Ceil_FN,
	Floor_FN,
	Trunc_FN,
	Nearest_FN,
	Sqrt_FN,
	Wrap_I32_I64,
	Trunc_I32_F32,
	Trunc_U32_F32,
	Trunc_I32_F64,
	Trunc_U32_F64,
	Extend_I32_I8,
	Extend_I32_I16,
	Extend_I64_I8,
	Extend_I64_I16,
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
			Self::Abs_FN => ("abs", "num"),
			Self::Neg_FN => ("neg", "num"),
			Self::Ceil_FN => ("ceil", "num"),
			Self::Floor_FN => ("floor", "num"),
			Self::Trunc_FN => ("trunc", "num"),
			Self::Nearest_FN => ("nearest", "num"),
			Self::Sqrt_FN => ("sqrt", "num"),
			Self::Wrap_I32_I64 => ("wrap", "i32_i64"),
			Self::Trunc_I32_F32 => ("trunc", "i32_f32"),
			Self::Trunc_U32_F32 => ("trunc", "u32_f32"),
			Self::Trunc_I32_F64 => ("trunc", "i32_f64"),
			Self::Trunc_U32_F64 => ("trunc", "u32_f64"),
			Self::Extend_I32_I8 => ("extend", "i32_i8"),
			Self::Extend_I32_I16 => ("extend", "i32_i16"),
			Self::Extend_I64_I8 => ("extend", "i64_i8"),
			Self::Extend_I64_I16 => ("extend", "i64_i16"),
			Self::Extend_I64_I32 => ("extend", "i64_i32"),
			Self::Extend_U64_I32 => ("extend", "u64_i32"),
			Self::Trunc_I64_F32 => ("trunc", "i64_f32"),
			Self::Trunc_U64_F32 => ("trunc", "u64_f32"),
			Self::Trunc_I64_F64 => ("trunc", "i64_f64"),
			Self::Trunc_U64_F64 => ("trunc", "u64_f64"),
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
			Operator::F32Abs | Operator::F64Abs => Self::Abs_FN,
			Operator::F32Neg | Operator::F64Neg => Self::Neg_FN,
			Operator::F32Ceil | Operator::F64Ceil => Self::Ceil_FN,
			Operator::F32Floor | Operator::F64Floor => Self::Floor_FN,
			Operator::F32Trunc | Operator::F64Trunc => Self::Trunc_FN,
			Operator::F32Nearest | Operator::F64Nearest => Self::Nearest_FN,
			Operator::F32Sqrt | Operator::F64Sqrt => Self::Sqrt_FN,
			Operator::I32WrapI64 => Self::Wrap_I32_I64,
			Operator::I32TruncF32S => Self::Trunc_I32_F32,
			Operator::I32TruncF32U => Self::Trunc_U32_F32,
			Operator::I32TruncF64S => Self::Trunc_I32_F64,
			Operator::I32TruncF64U => Self::Trunc_U32_F64,
			Operator::I32Extend8S => Self::Extend_I32_I8,
			Operator::I32Extend16S => Self::Extend_I32_I16,
			Operator::I64Extend8S => Self::Extend_I64_I8,
			Operator::I64Extend16S => Self::Extend_I64_I16,
			Operator::I64Extend32S | Operator::I64ExtendI32S => Self::Extend_I64_I32,
			Operator::I64ExtendI32U => Self::Extend_U64_I32,
			Operator::I64TruncF32S => Self::Trunc_I64_F32,
			Operator::I64TruncF32U => Self::Trunc_U64_F32,
			Operator::I64TruncF64S => Self::Trunc_I64_F64,
			Operator::I64TruncF64U => Self::Trunc_U64_F64,
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
	Add_FN,
	Sub_FN,
	Mul_FN,
	Div_FN,
	Min_FN,
	Max_FN,
	Copysign_FN,
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
			Self::Add_FN => ("add", "num"),
			Self::Sub_FN => ("sub", "num"),
			Self::Mul_FN => ("mul", "num"),
			Self::Div_FN => ("div", "num"),
			Self::Min_FN => ("min", "num"),
			Self::Max_FN => ("max", "num"),
			Self::Copysign_FN => ("copysign", "num"),
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
			Operator::F32Add | Operator::F64Add => Self::Add_FN,
			Operator::F32Sub | Operator::F64Sub => Self::Sub_FN,
			Operator::F32Mul | Operator::F64Mul => Self::Mul_FN,
			Operator::F32Div | Operator::F64Div => Self::Div_FN,
			Operator::F32Min | Operator::F64Min => Self::Min_FN,
			Operator::F32Max | Operator::F64Max => Self::Max_FN,
			Operator::F32Copysign | Operator::F64Copysign => Self::Copysign_FN,
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
	Eq_FN,
	Ne_FN,
	Lt_FN,
	Gt_FN,
	Le_FN,
	Ge_FN,
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
			Self::Eq_FN => ("eq", "num"),
			Self::Ne_FN => ("ne", "num"),
			Self::Lt_FN => ("lt", "num"),
			Self::Gt_FN => ("gt", "num"),
			Self::Le_FN => ("le", "num"),
			Self::Ge_FN => ("ge", "num"),
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
			Operator::F32Eq | Operator::F64Eq => Self::Eq_FN,
			Operator::F32Ne | Operator::F64Ne => Self::Ne_FN,
			Operator::F32Lt | Operator::F64Lt => Self::Lt_FN,
			Operator::F32Gt | Operator::F64Gt => Self::Gt_FN,
			Operator::F32Le | Operator::F64Le => Self::Le_FN,
			Operator::F32Ge | Operator::F64Ge => Self::Ge_FN,
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
	pub(crate) condition: Expression,
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

pub enum Terminator {
	Unreachable,
	Br(Br),
	BrTable(BrTable),
}

#[derive(Default)]
pub struct Forward {
	pub(crate) code: Vec<Statement>,
	pub(crate) last: Option<Terminator>,
}

impl Forward {
	#[must_use]
	pub fn code(&self) -> &[Statement] {
		&self.code
	}

	#[must_use]
	pub fn last(&self) -> Option<&Terminator> {
		self.last.as_ref()
	}
}

#[derive(Default)]
pub struct Backward {
	pub(crate) code: Vec<Statement>,
	pub(crate) last: Option<Terminator>,
}

impl Backward {
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
	pub(crate) condition: Expression,
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
	pub(crate) condition: Expression,
	pub(crate) on_true: Forward,
	pub(crate) on_false: Option<Forward>,
}

impl If {
	#[must_use]
	pub fn condition(&self) -> &Expression {
		&self.condition
	}

	#[must_use]
	pub fn on_true(&self) -> &Forward {
		&self.on_true
	}

	#[must_use]
	pub fn on_false(&self) -> Option<&Forward> {
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
	pub(crate) index: Expression,
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
	pub(crate) value: Expression,
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
	pub(crate) value: Expression,
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
	pub(crate) value: Expression,
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
	pub(crate) pointer: Expression,
	pub(crate) value: Expression,
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

pub enum Statement {
	Forward(Forward),
	Backward(Backward),
	BrIf(BrIf),
	If(If),
	Call(Call),
	CallIndirect(CallIndirect),
	SetTemporary(SetTemporary),
	SetLocal(SetLocal),
	SetGlobal(SetGlobal),
	StoreAt(StoreAt),
	MemoryGrow(MemoryGrow),
}

pub struct FuncData {
	pub(crate) local_data: Vec<(u32, ValType)>,
	pub(crate) num_result: usize,
	pub(crate) num_param: usize,
	pub(crate) num_stack: usize,
	pub(crate) code: Forward,
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
	pub fn code(&self) -> &Forward {
		&self.code
	}
}
