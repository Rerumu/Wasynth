use std::io::{Result, Write};

use parity_wasm::{deserialize_file, elements::Module};
use wasm_ast::builder::TypeInfo;

type Translator = fn(&Module, &TypeInfo, &mut dyn Write) -> Result<()>;

fn parse_module(name: &str) -> Module {
	let wasm = deserialize_file(name).expect("Failed to parse Wasm file");

	wasm.parse_names().unwrap_or_else(|v| v.1)
}

fn run_translator(wasm: &Module, runtime: &str, translator: Translator) {
	let mut pipe = std::io::stdout().lock();
	let type_info = TypeInfo::from_module(wasm);

	write!(pipe, "local rt = (function() {runtime} end)() ").unwrap();
	translator(wasm, &type_info, &mut pipe).unwrap();
}

fn do_translate(name: &str, file: &str) {
	let wasm = &parse_module(file);

	match name.to_lowercase().as_str() {
		"luajit" => run_translator(wasm, codegen_luajit::RUNTIME, codegen_luajit::translate),
		"luau" => run_translator(wasm, codegen_luau::RUNTIME, codegen_luau::translate),
		_ => panic!("Bad language: {name}"),
	}
}

fn do_help() {
	println!("usage: program translate <lang> <file>");
	println!("  or:  program help");
}

fn main() {
	let mut args = std::env::args().skip(1);

	match args.next().as_deref().unwrap_or("help") {
		"help" => do_help(),
		"translate" => {
			let lang = args.next().expect("No language specified");
			let file = args.next().expect("No file specified");

			do_translate(&lang, &file);
		}
		bad => {
			eprintln!("Bad action `{bad}`; try `help`");
		}
	}
}
