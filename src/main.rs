use backend::{
	helper::edition::{Edition, LuaJIT, Luau},
	translation::level_3,
};
use data::Module;
use parity_wasm::elements::deserialize_file;

mod backend;
mod data;

fn main() {
	let mut args = std::env::args().skip(1);
	let spec: &dyn Edition = match args.next().as_deref().map(str::to_lowercase).as_deref() {
		Some("luau") => &Luau,
		Some("luajit") => &LuaJIT,
		_ => {
			println!("expected either 'luau' or 'luajit' option");
			return;
		}
	};

	let output = std::io::stdout();

	for v in args {
		let wasm = deserialize_file(v).unwrap();
		let module = Module::new(&wasm);

		level_3::translate(spec, &module, &mut output.lock()).unwrap();
	}
}
