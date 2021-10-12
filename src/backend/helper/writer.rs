use std::io::{Result, Write};

pub type Writer<'a> = &'a mut dyn Write;

pub fn ordered_iter(prefix: &'static str, end: u32) -> impl Iterator<Item = String> {
	(1..=end).map(move |i| format!("{}_{}", prefix, i))
}

pub fn write_ordered(prefix: &'static str, end: u32, w: Writer) -> Result<()> {
	let mut iter = ordered_iter(prefix, end);

	if let Some(s) = iter.next() {
		write!(w, "{}", s)?;
	}

	iter.try_for_each(|s| write!(w, ", {}", s))
}
