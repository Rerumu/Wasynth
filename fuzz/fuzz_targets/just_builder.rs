#![no_main]

use parity_wasm::elements::Module as WasmModule;
use wasm_smith::Module as SmModule;

use codegen_luajit::gen::Generator;
use wasm_ast::writer::Transpiler;

fn fuzz_transformer(wasm: &WasmModule) {
	let trans = Generator::new(wasm);
	let _func = trans.build_func_list();
}

libfuzzer_sys::fuzz_target!(|module: SmModule| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	fuzz_transformer(&wasm);
});
