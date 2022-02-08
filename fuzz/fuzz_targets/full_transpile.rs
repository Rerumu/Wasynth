#![no_main]

use wasm_smith::Module;

use codegen_luajit::gen::Generator;
use wasm_ast::writer::Transpiler;

// We are not interested in parity_wasm errors.
// Only 1 edition should need to be tested too.
libfuzzer_sys::fuzz_target!(|module: Module| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	Generator::new(&wasm)
		.transpile(&mut std::io::sink())
		.expect("LuaJIT should succeed");
});
