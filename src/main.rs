use backend::{edition::data::from_string, translation::level_3};
use data::Module;
use parity_wasm::elements::deserialize_file;

mod backend;
mod data;

fn main() {
	let mut args = std::env::args().skip(1);
	let spec = args
		.next()
		.as_deref()
		.and_then(from_string)
		.expect("No language argument provided");

	let output = std::io::stdout();

	for v in args {
		let wasm = deserialize_file(v).unwrap();
		let module = Module::new(&wasm);

		level_3::translate(spec, &module, &mut output.lock()).unwrap();
	}
}
