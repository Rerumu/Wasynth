#![cfg(test)]

use std::{
	io::{Result as IResult, Write},
	marker::PhantomData,
	path::{Path, PathBuf},
	process::Command,
};

use parity_wasm::elements::Module as BinModule;
use wast::{
	core::Module as AstModule, parser::ParseBuffer, token::Id, QuoteWat, Wast, WastDirective, Wat,
};

use wasm_ast::builder::TypeInfo;

struct TypedModule<'a> {
	name: &'a str,
	module: &'a BinModule,
	type_info: TypeInfo<'a>,
}

impl<'a> TypedModule<'a> {
	fn resolve_id(id: Option<Id>) -> &str {
		id.map_or("temp", |v| v.name())
	}

	fn from_id(id: Option<Id<'a>>, module: &'a BinModule) -> Self {
		Self {
			module,
			name: Self::resolve_id(id),
			type_info: TypeInfo::from_module(module),
		}
	}
}

trait Target: Sized {
	fn executable() -> String;

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> IResult<()>;

	fn write_runtime(w: &mut dyn Write) -> IResult<()>;

	fn write_module(data: TypedModule, w: &mut dyn Write) -> IResult<()>;
}

struct LuaJIT;

impl Target for LuaJIT {
	fn executable() -> String {
		std::env::var("LUAJIT_PATH").unwrap_or_else(|_| "luajit".to_string())
	}

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> IResult<()> {
		writeln!(w, "local {} = {}", post, pre)
	}

	fn write_runtime(w: &mut dyn Write) -> IResult<()> {
		let runtime = codegen_luajit::RUNTIME;

		writeln!(w, "local rt = (function() {runtime} end)()")
	}

	fn write_module(data: TypedModule, w: &mut dyn Write) -> IResult<()> {
		write!(w, "local temp_{} = (function() ", data.name)?;
		codegen_luajit::translate(data.module, &data.type_info, w)?;
		writeln!(w, "end)(nil)")
	}
}

struct Luau;

impl Target for Luau {
	fn executable() -> String {
		std::env::var("LUAU_PATH").unwrap_or_else(|_| "luajit".to_string())
	}

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> IResult<()> {
		writeln!(w, "local {} = {}", post, pre)
	}

	fn write_runtime(w: &mut dyn Write) -> IResult<()> {
		let runtime = codegen_luau::RUNTIME;

		writeln!(w, "local rt = (function() {runtime} end)()")
	}

	fn write_module(data: TypedModule, w: &mut dyn Write) -> IResult<()> {
		write!(w, "local temp_{} = (function() ", data.name)?;
		codegen_luau::translate(data.module, &data.type_info, w)?;
		writeln!(w, "end)(nil)")
	}
}

fn try_into_ast_module(data: QuoteWat) -> Option<AstModule> {
	if let QuoteWat::Wat(Wat::Module(data)) = data {
		Some(data)
	} else {
		None
	}
}

// Only proceed with tests that observe any state.
fn parse_and_validate<'a>(buffer: &'a ParseBuffer) -> Option<Wast<'a>> {
	let loaded: Wast = wast::parser::parse(buffer).unwrap();
	let observer = loaded.directives.iter().any(|v| {
		matches!(
			v,
			WastDirective::AssertTrap { .. }
				| WastDirective::AssertReturn { .. }
				| WastDirective::AssertExhaustion { .. }
		)
	});

	observer.then(|| loaded)
}

struct Tester<T> {
	_marker: PhantomData<T>,
}

impl<T: Target> Tester<T> {
	fn test(name: &str, source: &str) -> IResult<()> {
		if let Some(data) = Self::run_generation(source)? {
			let temp = std::env::temp_dir().join("wasm-test-".to_string() + name);

			std::fs::write(&temp, &data)?;
			Self::run_command(&temp)?;
		}

		Ok(())
	}

	fn write_variant(variant: WastDirective, w: &mut dyn Write) -> IResult<()> {
		match variant {
			WastDirective::Wat(data) => {
				let mut ast = try_into_ast_module(data).expect("Must be a module");
				let bytes = ast.encode().unwrap();
				let temp = parity_wasm::deserialize_buffer(&bytes).unwrap();
				let data = TypedModule::from_id(ast.id, &temp);

				T::write_module(data, w)
			}
			WastDirective::Register { name, module, .. } => {
				T::write_register(name, TypedModule::resolve_id(module), w)
			}
			// WastDirective::Invoke(_) => todo!(),
			// WastDirective::AssertTrap { exec, message, .. } => todo!(),
			// WastDirective::AssertReturn { exec, results, .. } => todo!(),
			// WastDirective::AssertExhaustion { call, message, .. } => todo!(),
			_ => Ok(()),
		}
	}

	fn run_command(file: &Path) -> IResult<()> {
		let result = Command::new(T::executable()).arg(file).output()?;

		if result.status.success() {
			Ok(())
		} else {
			let data = String::from_utf8_lossy(&result.stderr);

			panic!("{}", data);
		}
	}

	fn run_generation(source: &str) -> IResult<Option<Vec<u8>>> {
		let lexed = ParseBuffer::new(source).expect("Failed to tokenize");
		let parsed = match parse_and_validate(&lexed) {
			Some(v) => v,
			None => return Ok(None),
		};

		let mut data = Vec::new();

		T::write_runtime(&mut data)?;

		for variant in parsed.directives {
			Self::write_variant(variant, &mut data)?;
		}

		Ok(Some(data))
	}
}

static DO_NOT_RUN: [&str; 3] = ["binary-leb128.wast", "conversions.wast", "names.wast"];

#[test_generator::test_resources("dev-test/spec/*.wast")]
fn translate_file(path: PathBuf) {
	let path = path.strip_prefix("dev-test/").unwrap();
	let name = path.file_name().unwrap().to_str().unwrap();

	if DO_NOT_RUN.contains(&name) {
		return;
	}

	let source = std::fs::read_to_string(path).unwrap();

	Tester::<LuaJIT>::test(name, &source).unwrap();
	// Tester::<Luau>::test(name, &source).unwrap();
}
