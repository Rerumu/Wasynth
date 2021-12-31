use std::io::{Result, Write};

use parity_wasm::elements::Module;

pub type Writer<'a> = &'a mut dyn Write;

pub trait Transpiler<'a> {
	fn new(wasm: &'a Module) -> Self
	where
		Self: Sized;

	fn runtime(writer: Writer) -> Result<()>;

	/// # Errors
	/// Returns `Err` if writing to `Writer` failed.
	fn transpile(&self, writer: Writer) -> Result<()>;
}
