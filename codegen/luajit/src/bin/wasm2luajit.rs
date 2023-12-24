use std::io::{ErrorKind, Result, Write};

use wasm_ast::module::Module;

fn load_arg_source() -> Result<Vec<u8>> {
	let mut arguments = std::env::args();
	let path = arguments
		.next()
		.unwrap_or_else(|| "wasm2luajit".to_string());

	arguments.next().map_or_else(
		|| {
			eprintln!("usage: {path} <file>\n");

			Err(ErrorKind::NotFound.into())
		},
		std::fs::read,
	)
}

fn do_runtime(lock: &mut dyn Write) -> Result<()> {
	let runtime = codegen_luajit::RUNTIME;

	writeln!(lock, "local rt = (function()")?;
	writeln!(lock, "{runtime}")?;
	writeln!(lock, "end)()")
}

fn main() -> Result<()> {
	let data = load_arg_source()?;
	let wasm = Module::try_from_data(&data).unwrap();

	let lock = &mut std::io::stdout().lock();

	do_runtime(lock)?;
	codegen_luajit::from_module_untyped(&wasm, lock)
}
