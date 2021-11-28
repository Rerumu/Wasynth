#![no_main]

use std::io::Result;

use parity_wasm::elements::Module as WasmModule;
use wasm_smith::Module as SmModule;

use wasm::backend::{
	edition::{data::Edition, luajit::LuaJIT},
	translator::data::Module,
};

fn fuzz_translate(wasm: &WasmModule, ed: &dyn Edition) -> Result<()> {
	let mut sink = std::io::sink();
	let module = Module::new(wasm);

	module.translate(ed, &mut sink)
}

// We are not interested in parity_wasm errors.
// Only 1 edition should need to be tested too.
libfuzzer_sys::fuzz_target!(|module: SmModule| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	fuzz_translate(&wasm, &LuaJIT).expect("LuaJIT should succeed");
});
