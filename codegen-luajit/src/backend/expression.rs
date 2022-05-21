use std::io::{Result, Write};

use wasm_ast::node::{
	BinOp, CmpOp, Expression, GetGlobal, GetLocal, LoadAt, MemoryGrow, MemorySize, Recall, Select,
	UnOp, Value,
};

use crate::analyzer::operator::bin_symbol_of;

use super::manager::{
	write_cmp_op, write_condition, write_separated, write_variable, Driver, Manager,
};

impl Driver for Recall {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} ", self.var)
	}
}

impl Driver for Select {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		write_condition(&self.cond, mng, w)?;
		write!(w, "and ")?;
		self.a.write(mng, w)?;
		write!(w, "or ")?;
		self.b.write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for GetLocal {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_variable(self.var, mng, w)
	}
}

impl Driver for GetGlobal {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value ", self.var)
	}
}

impl Driver for LoadAt {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "load_{}(memory_at_0, ", self.what.as_name())?;
		self.pointer.write(mng, w)?;

		if self.offset != 0 {
			write!(w, "+ {}", self.offset)?;
		}

		write!(w, ")")
	}
}

impl Driver for MemorySize {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "memory_at_{}.min ", self.memory)
	}
}

impl Driver for MemoryGrow {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "rt.allocator.grow(memory_at_{}, ", self.memory)?;
		self.value.write(mng, w)?;
		write!(w, ")")
	}
}

fn write_f32(number: f32, w: &mut dyn Write) -> Result<()> {
	let sign = if number.is_sign_negative() { "-" } else { "" };

	if number.is_infinite() {
		write!(w, "{sign}math.huge ")
	} else if number.is_nan() {
		write!(w, "{sign}0/0 ")
	} else {
		write!(w, "{number:e} ")
	}
}

fn write_f64(number: f64, w: &mut dyn Write) -> Result<()> {
	let sign = if number.is_sign_negative() { "-" } else { "" };

	if number.is_infinite() {
		write!(w, "{sign}math.huge ")
	} else if number.is_nan() {
		write!(w, "{sign}0/0 ")
	} else {
		write!(w, "{number:e} ")
	}
}

impl Driver for Value {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::I32(i) => write!(w, "{i} "),
			Self::I64(i) => write!(w, "{i}LL "),
			Self::F32(f) => write_f32(*f, w),
			Self::F64(f) => write_f64(*f, w),
		}
	}
}

impl Driver for UnOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let (a, b) = self.op.as_name();

		write!(w, "{a}_{b}(")?;
		self.rhs.write(mng, w)?;
		write!(w, ")")
	}
}

impl Driver for BinOp {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if let Some(symbol) = bin_symbol_of(self.op) {
			write!(w, "(")?;
			self.lhs.write(mng, w)?;
			write!(w, "{symbol} ")?;
			self.rhs.write(mng, w)?;
			write!(w, ")")
		} else {
			let (head, tail) = self.op.as_name();

			write!(w, "{head}_{tail}(")?;
			self.lhs.write(mng, w)?;
			write!(w, ", ")?;
			self.rhs.write(mng, w)?;
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
			Self::Recall(e) => e.write(mng, w),
			Self::Select(e) => e.write(mng, w),
			Self::GetLocal(e) => e.write(mng, w),
			Self::GetGlobal(e) => e.write(mng, w),
			Self::LoadAt(e) => e.write(mng, w),
			Self::MemorySize(e) => e.write(mng, w),
			Self::MemoryGrow(e) => e.write(mng, w),
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
