#![no_main]

use wasm_ast::builder::TypeInfo;
use wasm_smith::Module;

// We are not interested in parity_wasm errors.
libfuzzer_sys::fuzz_target!(|module: Module| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	let type_info = TypeInfo::from_module(&wasm);
	let sink = &mut std::io::sink();

	codegen_luau::from_module(&wasm, &type_info, sink).expect("Luau should succeed");
});
