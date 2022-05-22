#![no_main]

use wasm_smith::Module;

// We are not interested in parity_wasm errors.
libfuzzer_sys::fuzz_target!(|module: Module| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	let sink = &mut std::io::sink();

	codegen_luajit::from_module_untyped(&wasm, sink).expect("LuaJIT should succeed");
});
