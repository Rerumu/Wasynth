use backend::{edition::data::from_string, translator::data::Module};
use parity_wasm::deserialize_file;

mod backend;

fn main() {
	let mut args = std::env::args().skip(1);
	let ed = args
		.next()
		.as_deref()
		.and_then(from_string)
		.expect("No language argument provided");

	let output = std::io::stdout();

	for v in args {
		let wasm = deserialize_file(v).unwrap();
		let module = Module::new(&wasm);

		module.translate(ed, &mut output.lock()).unwrap();
	}
}
