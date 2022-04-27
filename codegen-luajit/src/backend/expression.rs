use std::io::{Result, Write};

use wasm_ast::node::{
	BinOp, CmpOp, Expression, GetGlobal, GetLocal, LoadAt, MemoryGrow, MemorySize, Recall, Select,
	UnOp, Value,
};

use super::manager::{
	write_bin_call, write_cmp_op, write_condition, write_f32, write_f64, write_separated,
	write_variable, Driver, Manager,
};

impl Driver for Recall {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "reg_{} ", self.var)
	}
}

impl Driver for Select {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		write_condition(&self.cond, v, w)?;
		write!(w, "and ")?;
		self.a.write(v, w)?;
		write!(w, "or ")?;
		self.b.write(v, w)?;
		write!(w, ")")
	}
}

impl Driver for GetLocal {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_variable(self.var, v, w)
	}
}

impl Driver for GetGlobal {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "GLOBAL_LIST[{}].value ", self.var)
	}
}

impl Driver for LoadAt {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "load_{}(memory_at_0, ", self.what.as_name())?;
		self.pointer.write(v, w)?;
		write!(w, "+ {})", self.offset)
	}
}

impl Driver for MemorySize {
	fn write(&self, _: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "memory_at_{}.min ", self.memory)
	}
}

impl Driver for MemoryGrow {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "rt.allocator.grow(memory_at_{}, ", self.memory)?;
		self.value.write(v, w)?;
		write!(w, ")")
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
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		let (a, b) = self.op.as_name();

		write!(w, "{a}_{b}(")?;
		self.rhs.write(v, w)?;
		write!(w, ")")
	}
}

impl Driver for BinOp {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		if let Some(op) = self.op.as_operator() {
			write!(w, "(")?;
			self.lhs.write(v, w)?;
			write!(w, "{op} ")?;
			self.rhs.write(v, w)?;
			write!(w, ")")
		} else {
			write_bin_call(self.op.as_name(), &self.lhs, &self.rhs, v, w)
		}
	}
}

impl Driver for CmpOp {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write!(w, "(")?;
		write_cmp_op(self, v, w)?;
		write!(w, "and 1 or 0)")
	}
}

impl Driver for Expression {
	fn write(&self, v: &mut Manager, w: &mut dyn Write) -> Result<()> {
		match self {
			Self::Recall(e) => e.write(v, w),
			Self::Select(e) => e.write(v, w),
			Self::GetLocal(e) => e.write(v, w),
			Self::GetGlobal(e) => e.write(v, w),
			Self::LoadAt(e) => e.write(v, w),
			Self::MemorySize(e) => e.write(v, w),
			Self::MemoryGrow(e) => e.write(v, w),
			Self::Value(e) => e.write(v, w),
			Self::UnOp(e) => e.write(v, w),
			Self::BinOp(e) => e.write(v, w),
			Self::CmpOp(e) => e.write(v, w),
		}
	}
}

impl Driver for &[Expression] {
	fn write(&self, mng: &mut Manager, w: &mut dyn Write) -> Result<()> {
		write_separated(self.iter(), |e, w| e.write(mng, w), w)
	}
}
