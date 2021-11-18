use std::io::Result;

use crate::backend::helper::writer::Writer;

use super::data::{Edition, Infix};

pub struct Luau;

impl Edition for Luau {
	fn runtime(&self) -> &'static str {
		"script.Runtime"
	}

	fn start_block(&self, w: Writer) -> Result<()> {
		write!(w, "while true do ")
	}

	fn start_loop(&self, _level: usize, w: Writer) -> Result<()> {
		write!(w, "while true do ")
	}

	fn start_if(&self, cond: &str, w: Writer) -> Result<()> {
		write!(w, "while true do ")?;
		write!(w, "if {} ~= 0 then ", cond)
	}

	fn end_block(&self, _level: usize, w: Writer) -> Result<()> {
		write!(w, "break ")?;
		write!(w, "end ")
	}

	fn end_loop(&self, w: Writer) -> Result<()> {
		write!(w, "break ")?;
		write!(w, "end ")
	}

	fn end_if(&self, _level: usize, w: Writer) -> Result<()> {
		write!(w, "end ")?;
		write!(w, "break ")?;
		write!(w, "end ")
	}

	fn br_target(&self, level: usize, in_loop: bool, w: Writer) -> Result<()> {
		write!(w, "if desired then ")?;
		write!(w, "if desired == {} then ", level)?;
		write!(w, "desired = nil ")?;

		if in_loop {
			write!(w, "continue ")?;
		}

		write!(w, "end ")?;
		write!(w, "break ")?;
		write!(w, "end ")
	}

	fn br_to_level(&self, level: usize, up: usize, is_loop: bool, w: Writer) -> Result<()> {
		write!(w, "do ")?;

		if up == 0 {
			if is_loop {
				write!(w, "continue ")?;
			} else {
				write!(w, "break ")?;
			}
		} else {
			write!(w, "desired = {} ", level - up)?;
			write!(w, "break ")?;
		}

		write!(w, "end ")
	}

	fn i64(&self, i: i64) -> Infix<i64> {
		Infix::new("", i)
	}
}
