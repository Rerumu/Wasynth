use std::io::{Result, Write};

use parity_wasm::{deserialize_file, elements::Module};

type FromUntyped = fn(&Module, &mut dyn Write) -> Result<()>;

fn run_with(file: &str, runtime: &str, from_untyped: FromUntyped) -> Result<()> {
	let wasm = deserialize_file(file)
		.expect("Failed to parse Wasm file")
		.parse_names()
		.unwrap_or_else(|v| v.1);

	let lock = &mut std::io::stdout().lock();

	write!(lock, "local rt = (function() {runtime} end)() ")?;
	from_untyped(&wasm, lock)
}

fn do_translate(lang: &str, file: &str) {
	let result = match lang.to_lowercase().as_str() {
		"luajit" => run_with(
			file,
			codegen_luajit::RUNTIME,
			codegen_luajit::from_module_untyped,
		),
		"luau" => run_with(
			file,
			codegen_luau::RUNTIME,
			codegen_luau::from_module_untyped,
		),
		_ => panic!("Bad language: {lang}"),
	};

	result.expect("Failed to translate file");
}

fn do_help() {
	println!("usage: program to <lang> <file>");
	println!("  or:  program help");
}

fn main() {
	let mut args = std::env::args().skip(1);

	match args.next().as_deref().unwrap_or("help") {
		"help" => do_help(),
		"to" => {
			let lang = args.next().expect("No language specified");
			let file = args.next().expect("No file specified");

			do_translate(&lang, &file);
		}
		bad => {
			eprintln!("Bad action `{bad}`; try `help`");
		}
	}
}
