use std::{
	io::{Result, Write},
	num::FpCategory,
};

use wasm_ast::node::{
	BinOp, CmpOp, Expression, GetGlobal, LoadAt, Local, MemorySize, Select, Temporary, UnOp, Value,
};

use crate::analyzer::as_symbol::AsSymbol;

use super::manager::{write_separated, Driver, Manager};

macro_rules! impl_write_number {
	($name:tt, $numeric:ty) => {
		fn $name(number: $numeric, w: &mut dyn Write) -> Result<()> {
			match (number.classify(), number.is_sign_negative()) {
				(FpCategory::Nan, true) => write!(w, "(0.0 / 0.0)"),
				(FpCategory::Nan, false) => write!(w, "-(0.0 / 0.0)"),
				(FpCategory::Infinite, true) => write!(w, "-math.huge"),
				(FpCategory::Infinite, false) => write!(w, "math.huge"),
				_ => write!(w, "{number:e}"),
			}
		}
	};
}

impl Driver for Select {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		Condition(self.condition()).write(mng, w)?;
		write!(w, " and ")?;
		self.on_true().write(mng, w)?;
		write!(w, " or ")?;
		self.on_false().write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for Temporary {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let var = self.var();

		if let Some(var) = var.checked_sub(mng.num_temp()) {
			write!(w, "reg_spill[{}]", var + 1)
		} else {
			write!(w, "reg_{var}")
		}
	}
}

impl Driver for Local {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let var = self.var();

		if let Some(var) = var.checked_sub(mng.num_local()) {
			write!(w, "loc_spill[{}]", var + 1)
		} else {
			write!(w, "loc_{var}")
		}
	}
}

impl Driver for GetGlobal {
	fn write(&self, _mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value", self.var())
	}
}

impl Driver for LoadAt {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let name = self.load_type().as_name();
		let memory = self.memory();

		write!(w, "load_{name}(memory_at_{memory}, ")?;
		self.pointer().write(mng, w)?;

		if self.offset() != 0 {
			write!(w, " + {}", self.offset())?;
		}

		write!(w, ")")
	}
}

impl Driver for MemorySize {
	fn write(&self, _mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "memory_at_{}.min", self.memory())
	}
}

impl_write_number!(write_f32, f32);
impl_write_number!(write_f64, f64);

impl Driver for Value {
	fn write(&self, _mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::I32(i) => write!(w, "{i}"),
			Self::I64(i) => write!(w, "{i}LL"),
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
			write!(w, " {symbol} ")?;
		} else {
			let (head, tail) = self.op_type().as_name();

			write!(w, "{head}_{tail}(")?;
			self.lhs().write(mng, w)?;
			write!(w, ", ")?;
		}

		self.rhs().write(mng, w)?;
		write!(w, ")")
	}
}

struct CmpOpBoolean<'a>(&'a CmpOp);

impl Driver for CmpOpBoolean<'_> {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let cmp = self.0;

		if let Some(symbol) = cmp.op_type().as_symbol() {
			cmp.lhs().write(mng, w)?;
			write!(w, " {symbol} ")?;
			cmp.rhs().write(mng, w)
		} else {
			let (head, tail) = cmp.op_type().as_name();

			write!(w, "{head}_{tail}(")?;
			cmp.lhs().write(mng, w)?;
			write!(w, ", ")?;
			cmp.rhs().write(mng, w)?;
			write!(w, ")")
		}
	}
}

impl Driver for CmpOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		CmpOpBoolean(self).write(mng, w)?;
		write!(w, " and 1 or 0)")
	}
}

pub struct Condition<'a>(pub &'a Expression);

impl Driver for Condition<'_> {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if let Expression::CmpOp(node) = self.0 {
			CmpOpBoolean(node).write(mng, w)
		} else {
			self.0.write(mng, w)?;
			write!(w, " ~= 0")
		}
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
