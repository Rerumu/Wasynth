use std::{
	io::{Result, Write},
	num::FpCategory,
};

use wasm_ast::node::{
	BinOp, CmpOp, Expression, GetGlobal, GetLocal, GetTemporary, LoadAt, MemorySize, Select, UnOp,
	Value,
};

use crate::analyzer::as_symbol::AsSymbol;

use super::manager::{
	write_cmp_op, write_condition, write_separated, write_variable, Driver, Manager,
};

macro_rules! impl_write_number {
	($name:tt, $numeric:ty) => {
		fn $name(number: $numeric, w: &mut dyn Write) -> Result<()> {
			match (number.classify(), number.is_sign_negative()) {
				(FpCategory::Nan, true) => write!(w, "(0.0 / 0.0) "),
				(FpCategory::Nan, false) => write!(w, "-(0.0 / 0.0) "),
				(FpCategory::Infinite, true) => write!(w, "-math.huge "),
				(FpCategory::Infinite, false) => write!(w, "math.huge "),
				_ => write!(w, "{number:e} "),
			}
		}
	};
}

impl Driver for Select {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		write_condition(self.condition(), mng, w)?;
		write!(w, "and ")?;
		self.on_true().write(mng, w)?;
		write!(w, "or ")?;
		self.on_false().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for GetTemporary {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} ", self.var())
	}
}

impl Driver for GetLocal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_variable(self.var(), mng, w)
	}
}

impl Driver for GetGlobal {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value ", self.var())
	}
}

impl Driver for LoadAt {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "load_{}(memory_at_0, ", self.load_type().as_name())?;
		self.pointer().write(mng, w)?;

		if self.offset() != 0 {
			write!(w, "+ {}", self.offset())?;
		}

		write!(w, ")")
	}
}

impl Driver for MemorySize {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "memory_at_{}.min ", self.memory())
	}
}

pub fn write_i32(number: i32, w: &mut dyn Write) -> Result<()> {
	let list = number.to_ne_bytes();

	write!(w, "{} ", u32::from_ne_bytes(list))
}

fn write_i64(number: i64, w: &mut dyn Write) -> Result<()> {
	match number {
		0 => write!(w, "i64_ZERO "),
		1 => write!(w, "i64_ONE "),
		_ => {
			let list = number.to_ne_bytes();
			let a = u32::from_ne_bytes(list[0..4].try_into().unwrap());
			let b = u32::from_ne_bytes(list[4..8].try_into().unwrap());

			write!(w, "i64_from_u32({a}, {b}) ")
		}
	}
}

impl_write_number!(write_f32, f32);
impl_write_number!(write_f64, f64);

impl Driver for Value {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::I32(i) => write_i32(*i, w),
			Self::I64(i) => write_i64(*i, w),
			Self::F32(f) => write_f32(*f, w),
			Self::F64(f) => write_f64(*f, w),
		}
	}
}

impl Driver for UnOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let (a, b) = self.op_type().as_name();

		write!(w, "{a}_{b}(")?;
		self.rhs().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for BinOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if let Some(symbol) = self.op_type().as_symbol() {
			write!(w, "(")?;
			self.lhs().write(mng, w)?;
			write!(w, "{symbol} ")?;
			self.rhs().write(mng, w)?;
			write!(w, ")")
		} else {
			let (head, tail) = self.op_type().as_name();

			write!(w, "{head}_{tail}(")?;
			self.lhs().write(mng, w)?;
			write!(w, ", ")?;
			self.rhs().write(mng, w)?;
			write!(w, ")")
		}
	}
}

impl Driver for CmpOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		write_cmp_op(self, mng, w)?;
		write!(w, "and 1 or 0)")
	}
}

impl Driver for Expression {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Select(e) => e.write(mng, w),
			Self::GetTemporary(e) => e.write(mng, w),
			Self::GetLocal(e) => e.write(mng, w),
			Self::GetGlobal(e) => e.write(mng, w),
			Self::LoadAt(e) => e.write(mng, w),
			Self::MemorySize(e) => e.write(mng, w),
			Self::Value(e) => e.write(mng, w),
			Self::UnOp(e) => e.write(mng, w),
			Self::BinOp(e) => e.write(mng, w),
			Self::CmpOp(e) => e.write(mng, w),
		}
	}
}

impl Driver for &[Expression] {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_separated(self.iter(), |e, w| e.write(mng, w), w)
	}
}
