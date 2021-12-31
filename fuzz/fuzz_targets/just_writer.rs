#![no_main]

use std::io::Result;

use parity_wasm::elements::Module as WasmModule;
use wasm_smith::Module as SmModule;

use wasm::writer::{base::Transpiler, luajit::LuaJIT};

fn fuzz_writer(wasm: &WasmModule) -> Result<()> {
	let trans = LuaJIT::new(wasm);
	let list = trans.build_func_list();

	trans.gen_func_list(&list, &mut std::io::sink())
}

libfuzzer_sys::fuzz_target!(|module: SmModule| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	fuzz_writer(&wasm).expect("LuaJIT should succeed");
});
