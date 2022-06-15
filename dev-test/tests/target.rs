use std::{
	io::{Result, Write},
	path::{Path, PathBuf},
	process::Command,
};

use parity_wasm::elements::Module as BinModule;
use wast::{
	core::Module as AstModule, parser::ParseBuffer, token::Id, AssertExpression, QuoteWat, Wast,
	WastDirective, WastExecute, WastInvoke, Wat,
};

use wasm_ast::builder::TypeInfo;

macro_rules! impl_write_number_nan {
	($name:ident, $name_nan:ident, $numeric:ty, $pattern:ty) => {
		pub fn $name(number: $numeric, w: &mut dyn Write) -> Result<()> {
			let sign = if number.is_sign_negative() { "-" } else { "" };

			if number.is_infinite() {
				write!(w, "{sign}LUA_INFINITY")
			} else if number.is_nan() {
				write!(w, "{sign}LUA_NAN_DEFAULT")
			} else {
				write!(w, "{number}")
			}
		}

		pub fn $name_nan(data: &wast::NanPattern<$pattern>, w: &mut dyn Write) -> Result<()> {
			use wast::NanPattern;

			match data {
				NanPattern::CanonicalNan => write!(w, "LUA_NAN_CANONICAL"),
				NanPattern::ArithmeticNan => write!(w, "LUA_NAN_ARITHMETIC"),
				NanPattern::Value(data) => {
					let number = <$numeric>::from_bits(data.bits);

					$name(number, w)
				}
			}
		}
	};
}

impl_write_number_nan!(write_f32, write_f32_nan, f32, wast::token::Float32);
impl_write_number_nan!(write_f64, write_f64_nan, f64, wast::token::Float64);

pub struct TypedModule<'a> {
	name: &'a str,
	module: &'a BinModule,
	type_info: TypeInfo<'a>,
}

impl<'a> TypedModule<'a> {
	pub fn resolve_id(id: Option<Id>) -> &str {
		id.map_or("temp", |v| v.name())
	}

	pub fn name(&self) -> &str {
		self.name
	}

	pub fn module(&self) -> &BinModule {
		self.module
	}

	pub fn type_info(&self) -> &TypeInfo<'a> {
		&self.type_info
	}

	fn from_id(id: Option<Id<'a>>, module: &'a BinModule) -> Self {
		Self {
			module,
			name: Self::resolve_id(id),
			type_info: TypeInfo::from_module(module),
		}
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
	let parsed: Wast = wast::parser::parse(buffer).unwrap();
	let observer = parsed.directives.iter().any(|v| {
		matches!(
			v,
			WastDirective::AssertTrap { .. }
				| WastDirective::AssertReturn { .. }
				| WastDirective::AssertExhaustion { .. }
		)
	});

	observer.then(|| parsed)
}

pub trait Target: Sized {
	fn executable() -> String;

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> Result<()>;

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> Result<()>;

	fn write_assert_trap(data: &WastExecute, w: &mut dyn Write) -> Result<()>;

	fn write_assert_return(
		data: &WastExecute,
		result: &[AssertExpression],
		w: &mut dyn Write,
	) -> Result<()>;

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> Result<()>;

	fn write_runtime(w: &mut dyn Write) -> Result<()>;

	fn write_module(typed: &TypedModule, w: &mut dyn Write) -> Result<()>;

	fn write_variant(variant: WastDirective, w: &mut dyn Write) -> Result<()> {
		match variant {
			WastDirective::Wat(data) => {
				let mut ast = try_into_ast_module(data).expect("Must be a module");
				let bytes = ast.encode().unwrap();
				let temp = parity_wasm::deserialize_buffer(&bytes).unwrap();
				let typed = TypedModule::from_id(ast.id, &temp);

				Self::write_module(&typed, w)?;
			}
			WastDirective::Register { name, module, .. } => {
				let pre = TypedModule::resolve_id(module);

				Self::write_register(name, pre, w)?;
			}
			WastDirective::Invoke(data) => {
				Self::write_invoke(&data, w)?;
			}
			WastDirective::AssertTrap { exec, .. } => {
				Self::write_assert_trap(&exec, w)?;
			}
			WastDirective::AssertReturn { exec, results, .. } => {
				Self::write_assert_return(&exec, &results, w)?;
			}
			WastDirective::AssertExhaustion { call, .. } => {
				Self::write_assert_exhaustion(&call, w)?;
			}
			_ => {}
		}

		Ok(())
	}

	fn run_command(file: &Path) -> Result<()> {
		let result = Command::new(Self::executable()).arg(file).output()?;

		if result.status.success() {
			Ok(())
		} else {
			let data = String::from_utf8_lossy(&result.stderr);

			panic!("{}", data);
		}
	}

	fn run_generation(source: &str) -> Result<Option<Vec<u8>>> {
		let lexed = ParseBuffer::new(source).expect("Failed to tokenize");
		let parsed = match parse_and_validate(&lexed) {
			Some(v) => v,
			None => return Ok(None),
		};

		let mut data = Vec::new();

		Self::write_runtime(&mut data)?;

		for variant in parsed.directives {
			Self::write_variant(variant, &mut data)?;
		}

		Ok(Some(data))
	}

	fn test(name: &str, source: &str) -> Result<()> {
		if let Some(data) = Self::run_generation(source)? {
			let temp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
				.join(name)
				.with_extension("wast.lua");

			std::fs::write(&temp, &data)?;
			Self::run_command(&temp)?;
		}

		Ok(())
	}
}
