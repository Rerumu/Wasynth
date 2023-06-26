#![no_main]

use wasm_ast::module::Module;
use wasm_smith::Module as RngModule;

libfuzzer_sys::fuzz_target!(|module: RngModule| {
	let data = module.to_bytes();
	let wasm = Module::try_from_data(&data).unwrap();

	let sink = &mut std::io::sink();

	codegen_luau::from_module_untyped(&wasm, sink).expect("Luau should succeed");
});
