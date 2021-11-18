use std::{fmt::Display, io::Result};

use crate::backend::helper::writer::Writer;

use super::{luajit::LuaJIT, luau::Luau};

pub struct Infix<T> {
	rhs: &'static str,
	inner: T,
}

impl<T> Infix<T> {
	pub fn new(rhs: &'static str, inner: T) -> Self {
		Infix { rhs, inner }
	}
}

impl<T> Display for Infix<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.inner.fmt(f)?;
		self.rhs.fmt(f)
	}
}

pub trait Edition {
	fn runtime(&self) -> &'static str;

	fn start_block(&self, w: Writer) -> Result<()>;
	fn start_loop(&self, level: usize, w: Writer) -> Result<()>;
	fn start_if(&self, cond: &str, w: Writer) -> Result<()>;
	fn end_block(&self, level: usize, w: Writer) -> Result<()>;
	fn end_loop(&self, w: Writer) -> Result<()>;
	fn end_if(&self, level: usize, w: Writer) -> Result<()>;

	fn br_target(&self, level: usize, in_loop: bool, w: Writer) -> Result<()>;
	fn br_to_level(&self, level: usize, up: usize, is_loop: bool, w: Writer) -> Result<()>;

	fn i64(&self, i: i64) -> Infix<i64>;
}

pub fn from_string(name: &str) -> Option<&'static dyn Edition> {
	match name.to_ascii_lowercase().as_str() {
		"luau" => Some(&Luau),
		"luajit" => Some(&LuaJIT),
		_ => None,
	}
}
