use parity_wasm::{deserialize_file, elements::Module};
use writer::{luajit::LuaJIT, luau::Luau, visit::Transpiler};

mod analyzer;
mod ast;
mod writer;

fn lang_from_string<'a>(name: &str, wasm: &'a Module) -> Box<dyn Transpiler<'a> + 'a> {
	match name.to_lowercase().as_str() {
		"luau" => Box::new(Luau::new(wasm)),
		"luajit" => Box::new(LuaJIT::new(wasm)),
		_ => panic!("Bad option: {}", name),
	}
}

fn parse_module(name: &str) -> Module {
	let wasm = deserialize_file(name).expect("Failed to parse Wasm file");

	match wasm.parse_names() {
		Ok(n) => n,
		Err(n) => n.1,
	}
}

fn main() {
	let mut args = std::env::args().skip(1);
	let name = args.next().expect("No language specified");

	let output = std::io::stdout();

	for v in args {
		let wasm = parse_module(&v);
		let module = lang_from_string(&name, &wasm);

		module.transpile(&mut output.lock()).unwrap();
	}
}
