#![no_main]

use std::io::Result;

use parity_wasm::elements::Module as WasmModule;
use wasm_smith::Module as SmModule;

use wasm::backend::{
	ast::transformer::Transformer,
	edition::{data::Edition, luajit::LuaJIT},
	translator::{arity::List, writer::Data},
};

fn fuzz_writer(wasm: &WasmModule, ed: &dyn Edition) -> Result<()> {
	let mut sink = std::io::sink();
	let arity = List::new(wasm);

	for i in 0..arity.in_arity.len() {
		let func = Transformer::new(wasm, &arity).consume(i);

		func.output(&mut Data::new(ed), &mut sink)?;
	}

	Ok(())
}

libfuzzer_sys::fuzz_target!(|module: SmModule| {
	let data = module.to_bytes();
	let wasm = match parity_wasm::deserialize_buffer(&data) {
		Ok(v) => v,
		Err(_) => return,
	};

	fuzz_writer(&wasm, &LuaJIT).expect("LuaJIT should succeed");
});
