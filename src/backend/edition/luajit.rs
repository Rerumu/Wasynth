use super::data::{Edition, Infix};
use crate::backend::helper::writer::Writer;
use std::io::Result;

pub struct LuaJIT;

impl Edition for LuaJIT {
	fn runtime(&self) -> &'static str {
		"'luajit'"
	}

	fn start_block(&self, w: Writer) -> Result<()> {
		write!(w, "do ")
	}

	fn start_loop(&self, level: usize, w: Writer) -> Result<()> {
		write!(w, "do ")?;
		write!(w, "::continue_at_{}::", level)
	}

	fn start_if(&self, cond: &str, w: Writer) -> Result<()> {
		write!(w, "if {} ~= 0 then ", cond)
	}

	fn end_block(&self, level: usize, w: Writer) -> Result<()> {
		write!(w, "::continue_at_{}::", level)?;
		write!(w, "end ")
	}

	fn end_loop(&self, w: Writer) -> Result<()> {
		write!(w, "end ")
	}

	fn end_if(&self, level: usize, w: Writer) -> Result<()> {
		write!(w, "::continue_at_{}::", level)?;
		write!(w, "end ")
	}

	fn br_target(&self, _level: usize, _in_loop: bool, _w: Writer) -> Result<()> {
		Ok(())
	}

	fn br_to_level(&self, level: usize, up: usize, _is_loop: bool, w: Writer) -> Result<()> {
		write!(w, "goto continue_at_{} ", level - up)
	}

	fn i64(&self, i: i64) -> Infix<i64> {
		Infix::new("LL", i)
	}
}
