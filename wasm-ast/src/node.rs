use std::ops::Range;

use parity_wasm::elements::{BrTableData, Instruction, Local, SignExtInstruction};

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

impl TryFrom<&Instruction> for LoadType {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		use Instruction as Inst;

		let result = match inst {
			Inst::I32Load(_, _) => Self::I32,
			Inst::I64Load(_, _) => Self::I64,
			Inst::F32Load(_, _) => Self::F32,
			Inst::F64Load(_, _) => Self::F64,
			Inst::I32Load8S(_, _) => Self::I32_I8,
			Inst::I32Load8U(_, _) => Self::I32_U8,
			Inst::I32Load16S(_, _) => Self::I32_I16,
			Inst::I32Load16U(_, _) => Self::I32_U16,
			Inst::I64Load8S(_, _) => Self::I64_I8,
			Inst::I64Load8U(_, _) => Self::I64_U8,
			Inst::I64Load16S(_, _) => Self::I64_I16,
			Inst::I64Load16U(_, _) => Self::I64_U16,
			Inst::I64Load32S(_, _) => Self::I64_I32,
			Inst::I64Load32U(_, _) => Self::I64_U32,
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

impl TryFrom<&Instruction> for StoreType {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		use Instruction as Inst;

		let result = match inst {
			Inst::I32Store(_, _) => Self::I32,
			Inst::I64Store(_, _) => Self::I64,
			Inst::F32Store(_, _) => Self::F32,
			Inst::F64Store(_, _) => Self::F64,
			Inst::I32Store8(_, _) => Self::I32_N8,
			Inst::I32Store16(_, _) => Self::I32_N16,
			Inst::I64Store8(_, _) => Self::I64_N8,
			Inst::I64Store16(_, _) => Self::I64_N16,
			Inst::I64Store32(_, _) => Self::I64_N32,
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

impl TryFrom<&Instruction> for UnOpType {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		use Instruction as Inst;

		let result = match inst {
			Inst::SignExt(ext) => match ext {
				SignExtInstruction::I32Extend8S => Self::Extend_I32_I8,
				SignExtInstruction::I32Extend16S => Self::Extend_I32_I16,
				SignExtInstruction::I64Extend8S => Self::Extend_I64_I8,
				SignExtInstruction::I64Extend16S => Self::Extend_I64_I16,
				SignExtInstruction::I64Extend32S => Self::Extend_I64_I32,
			},
			Inst::I32Clz => Self::Clz_I32,
			Inst::I32Ctz => Self::Ctz_I32,
			Inst::I32Popcnt => Self::Popcnt_I32,
			Inst::I64Clz => Self::Clz_I64,
			Inst::I64Ctz => Self::Ctz_I64,
			Inst::I64Popcnt => Self::Popcnt_I64,
			Inst::F32Abs | Inst::F64Abs => Self::Abs_FN,
			Inst::F32Neg | Inst::F64Neg => Self::Neg_FN,
			Inst::F32Ceil | Inst::F64Ceil => Self::Ceil_FN,
			Inst::F32Floor | Inst::F64Floor => Self::Floor_FN,
			Inst::F32Trunc | Inst::F64Trunc => Self::Trunc_FN,
			Inst::F32Nearest | Inst::F64Nearest => Self::Nearest_FN,
			Inst::F32Sqrt | Inst::F64Sqrt => Self::Sqrt_FN,
			Inst::I32WrapI64 => Self::Wrap_I32_I64,
			Inst::I32TruncSF32 => Self::Trunc_I32_F32,
			Inst::I32TruncUF32 => Self::Trunc_U32_F32,
			Inst::I32TruncSF64 => Self::Trunc_I32_F64,
			Inst::I32TruncUF64 => Self::Trunc_U32_F64,
			Inst::I64ExtendSI32 => Self::Extend_I64_I32,
			Inst::I64ExtendUI32 => Self::Extend_U64_I32,
			Inst::I64TruncSF32 => Self::Trunc_I64_F32,
			Inst::I64TruncUF32 => Self::Trunc_U64_F32,
			Inst::I64TruncSF64 => Self::Trunc_I64_F64,
			Inst::I64TruncUF64 => Self::Trunc_U64_F64,
			Inst::F32ConvertSI32 => Self::Convert_F32_I32,
			Inst::F32ConvertUI32 => Self::Convert_F32_U32,
			Inst::F32ConvertSI64 => Self::Convert_F32_I64,
			Inst::F32ConvertUI64 => Self::Convert_F32_U64,
			Inst::F32DemoteF64 => Self::Demote_F32_F64,
			Inst::F64ConvertSI32 => Self::Convert_F64_I32,
			Inst::F64ConvertUI32 => Self::Convert_F64_U32,
			Inst::F64ConvertSI64 => Self::Convert_F64_I64,
			Inst::F64ConvertUI64 => Self::Convert_F64_U64,
			Inst::F64PromoteF32 => Self::Promote_F64_F32,
			Inst::I32ReinterpretF32 => Self::Reinterpret_I32_F32,
			Inst::I64ReinterpretF64 => Self::Reinterpret_I64_F64,
			Inst::F32ReinterpretI32 => Self::Reinterpret_F32_I32,
			Inst::F64ReinterpretI64 => Self::Reinterpret_F64_I64,
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

impl TryFrom<&Instruction> for BinOpType {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		use Instruction as Inst;

		let result = match inst {
			Inst::I32Add => Self::Add_I32,
			Inst::I32Sub => Self::Sub_I32,
			Inst::I32Mul => Self::Mul_I32,
			Inst::I32DivS => Self::DivS_I32,
			Inst::I32DivU => Self::DivU_I32,
			Inst::I32RemS => Self::RemS_I32,
			Inst::I32RemU => Self::RemU_I32,
			Inst::I32And => Self::And_I32,
			Inst::I32Or => Self::Or_I32,
			Inst::I32Xor => Self::Xor_I32,
			Inst::I32Shl => Self::Shl_I32,
			Inst::I32ShrS => Self::ShrS_I32,
			Inst::I32ShrU => Self::ShrU_I32,
			Inst::I32Rotl => Self::Rotl_I32,
			Inst::I32Rotr => Self::Rotr_I32,
			Inst::I64Add => Self::Add_I64,
			Inst::I64Sub => Self::Sub_I64,
			Inst::I64Mul => Self::Mul_I64,
			Inst::I64DivS => Self::DivS_I64,
			Inst::I64DivU => Self::DivU_I64,
			Inst::I64RemS => Self::RemS_I64,
			Inst::I64RemU => Self::RemU_I64,
			Inst::I64And => Self::And_I64,
			Inst::I64Or => Self::Or_I64,
			Inst::I64Xor => Self::Xor_I64,
			Inst::I64Shl => Self::Shl_I64,
			Inst::I64ShrS => Self::ShrS_I64,
			Inst::I64ShrU => Self::ShrU_I64,
			Inst::I64Rotl => Self::Rotl_I64,
			Inst::I64Rotr => Self::Rotr_I64,
			Inst::F32Add | Inst::F64Add => Self::Add_FN,
			Inst::F32Sub | Inst::F64Sub => Self::Sub_FN,
			Inst::F32Mul | Inst::F64Mul => Self::Mul_FN,
			Inst::F32Div | Inst::F64Div => Self::Div_FN,
			Inst::F32Min | Inst::F64Min => Self::Min_FN,
			Inst::F32Max | Inst::F64Max => Self::Max_FN,
			Inst::F32Copysign | Inst::F64Copysign => Self::Copysign_FN,
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

impl TryFrom<&Instruction> for CmpOpType {
	type Error = ();

	fn try_from(inst: &Instruction) -> Result<Self, Self::Error> {
		use Instruction as Inst;

		let result = match inst {
			Inst::I32Eq => Self::Eq_I32,
			Inst::I32Ne => Self::Ne_I32,
			Inst::I32LtS => Self::LtS_I32,
			Inst::I32LtU => Self::LtU_I32,
			Inst::I32GtS => Self::GtS_I32,
			Inst::I32GtU => Self::GtU_I32,
			Inst::I32LeS => Self::LeS_I32,
			Inst::I32LeU => Self::LeU_I32,
			Inst::I32GeS => Self::GeS_I32,
			Inst::I32GeU => Self::GeU_I32,
			Inst::I64Eq => Self::Eq_I64,
			Inst::I64Ne => Self::Ne_I64,
			Inst::I64LtS => Self::LtS_I64,
			Inst::I64LtU => Self::LtU_I64,
			Inst::I64GtS => Self::GtS_I64,
			Inst::I64GtU => Self::GtU_I64,
			Inst::I64LeS => Self::LeS_I64,
			Inst::I64LeU => Self::LeU_I64,
			Inst::I64GeS => Self::GeS_I64,
			Inst::I64GeU => Self::GeU_I64,
			Inst::F32Eq | Inst::F64Eq => Self::Eq_FN,
			Inst::F32Ne | Inst::F64Ne => Self::Ne_FN,
			Inst::F32Lt | Inst::F64Lt => Self::Lt_FN,
			Inst::F32Gt | Inst::F64Gt => Self::Gt_FN,
			Inst::F32Le | Inst::F64Le => Self::Le_FN,
			Inst::F32Ge | Inst::F64Ge => Self::Ge_FN,
			_ => {
				return Err(());
			}
		};

		Ok(result)
	}
}

#[derive(Clone)]
pub struct GetTemporary {
	pub var: usize,
}

pub struct Select {
	pub cond: Box<Expression>,
	pub a: Box<Expression>,
	pub b: Box<Expression>,
}

pub struct GetLocal {
	pub var: usize,
}

pub struct GetGlobal {
	pub var: usize,
}

pub struct LoadAt {
	pub what: LoadType,
	pub offset: u32,
	pub pointer: Box<Expression>,
}

pub struct MemorySize {
	pub memory: usize,
}

pub struct MemoryGrow {
	pub memory: usize,
	pub value: Box<Expression>,
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
	pub op: UnOpType,
	pub rhs: Box<Expression>,
}

pub struct BinOp {
	pub op: BinOpType,
	pub lhs: Box<Expression>,
	pub rhs: Box<Expression>,
}

pub struct CmpOp {
	pub op: CmpOpType,
	pub lhs: Box<Expression>,
	pub rhs: Box<Expression>,
}

pub enum Expression {
	Select(Select),
	GetTemporary(GetTemporary),
	GetLocal(GetLocal),
	GetGlobal(GetGlobal),
	LoadAt(LoadAt),
	MemorySize(MemorySize),
	MemoryGrow(MemoryGrow),
	Value(Value),
	UnOp(UnOp),
	BinOp(BinOp),
	CmpOp(CmpOp),
}

impl Expression {
	#[must_use]
	pub fn has_side_effect(&self) -> bool {
		matches!(self, Expression::MemorySize(_))
	}

	#[must_use]
	pub fn is_temporary(&self, id: usize) -> bool {
		matches!(self, Expression::GetTemporary(v) if v.var == id)
	}

	#[must_use]
	pub fn is_local_read(&self, id: usize) -> bool {
		matches!(self, Expression::GetLocal(v) if v.var == id)
	}

	#[must_use]
	pub fn is_global_read(&self, id: usize) -> bool {
		matches!(self, Expression::GetGlobal(v) if v.var == id)
	}

	#[must_use]
	pub fn is_memory_size(&self, id: usize) -> bool {
		matches!(self, Expression::MemorySize(v) if v.memory == id)
	}

	#[must_use]
	pub fn is_memory_ref(&self, id: usize) -> bool {
		matches!(self, Expression::MemoryGrow(v) if v.memory == id)
			|| (id == 0 && matches!(self, Expression::LoadAt(_)))
	}
}

pub struct Br {
	pub target: usize,
}

pub struct BrTable {
	pub cond: Expression,
	pub data: BrTableData,
}

pub struct Return {
	pub list: Vec<Expression>,
}

pub enum Terminator {
	Unreachable,
	Br(Br),
	BrTable(BrTable),
	Return(Return),
}

#[derive(Default)]
pub struct Forward {
	pub code: Vec<Statement>,
	pub last: Option<Terminator>,
}

#[derive(Default)]
pub struct Backward {
	pub code: Vec<Statement>,
	pub last: Option<Terminator>,
}

pub struct If {
	pub cond: Expression,
	pub truthy: Forward,
	pub falsey: Option<Forward>,
}

pub struct BrIf {
	pub cond: Expression,
	pub target: usize,
}

pub struct Call {
	pub func: usize,
	pub result: Range<usize>,
	pub param_list: Vec<Expression>,
}

pub struct CallIndirect {
	pub table: usize,
	pub index: Expression,
	pub result: Range<usize>,
	pub param_list: Vec<Expression>,
}

pub struct SetTemporary {
	pub var: usize,
	pub value: Expression,
}

pub struct SetLocal {
	pub var: usize,
	pub value: Expression,
}

pub struct SetGlobal {
	pub var: usize,
	pub value: Expression,
}

pub struct StoreAt {
	pub what: StoreType,
	pub offset: u32,
	pub pointer: Expression,
	pub value: Expression,
}

pub enum Statement {
	Forward(Forward),
	Backward(Backward),
	If(If),
	BrIf(BrIf),
	Call(Call),
	CallIndirect(CallIndirect),
	SetTemporary(SetTemporary),
	SetLocal(SetLocal),
	SetGlobal(SetGlobal),
	StoreAt(StoreAt),
}

pub struct Intermediate {
	pub local_data: Vec<Local>,
	pub num_param: usize,
	pub num_stack: usize,
	pub code: Forward,
}
