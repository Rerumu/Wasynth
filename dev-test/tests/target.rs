use std::{
	io::{Result, Write},
	num::FpCategory,
	path::{Path, PathBuf},
	process::Command,
};

use wasm_ast::module::Module as AstModule;
use wast::{
	core::Module as WaModule, parser::ParseBuffer, token::Id, AssertExpression, QuoteWat, Wast,
	WastDirective, WastExecute, WastInvoke, Wat,
};

macro_rules! impl_write_number_nan {
	($name:ident, $name_nan:ident, $numeric:ty, $pattern:ty) => {
		pub fn $name(number: $numeric, w: &mut dyn Write) -> Result<()> {
			match (number.classify(), number.is_sign_negative()) {
				(FpCategory::Nan, true) => write!(w, "-LUA_NAN_DEFAULT "),
				(FpCategory::Nan, false) => write!(w, "LUA_NAN_DEFAULT "),
				(FpCategory::Infinite, true) => write!(w, "-math.huge "),
				(FpCategory::Infinite, false) => write!(w, "math.huge "),
				_ => write!(w, "{number:e} "),
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

fn try_into_ast_module(data: QuoteWat) -> Option<WaModule> {
	if let QuoteWat::Wat(Wat::Module(data)) = data {
		Some(data)
	} else {
		None
	}
}

pub fn get_name_from_id(id: Option<Id>) -> &str {
	id.as_ref().map_or("temp", Id::name)
}

pub trait Target: Sized {
	fn executable() -> String;

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> Result<()>;

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> Result<()>;

	fn write_assert_trap(data: &mut WastExecute, w: &mut dyn Write) -> Result<()>;

	fn write_assert_return(
		data: &mut WastExecute,
		result: &[AssertExpression],
		w: &mut dyn Write,
	) -> Result<()>;

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> Result<()>;

	fn write_runtime(w: &mut dyn Write) -> Result<()>;

	fn write_module(data: &AstModule, name: Option<&str>, w: &mut dyn Write) -> Result<()>;

	fn write_variant(variant: WastDirective, w: &mut dyn Write) -> Result<()> {
		match variant {
			WastDirective::Wat(data) => {
				let mut ast = try_into_ast_module(data).expect("Must be a module");
				let bytes = ast.encode().unwrap();

				let data = AstModule::from_data(&bytes);
				let name = ast.id.as_ref().map(Id::name);

				Self::write_module(&data, name, w)?;
			}
			WastDirective::Register { name, module, .. } => {
				let pre = get_name_from_id(module);

				Self::write_register(name, pre, w)?;
			}
			WastDirective::Invoke(data) => {
				Self::write_invoke(&data, w)?;
			}
			WastDirective::AssertTrap { mut exec, .. } => {
				Self::write_assert_trap(&mut exec, w)?;
			}
			WastDirective::AssertReturn {
				mut exec, results, ..
			} => {
				Self::write_assert_return(&mut exec, &results, w)?;
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

	fn run_generation(source: &str) -> Result<Vec<u8>> {
		let lexed = ParseBuffer::new(source).expect("Failed to tokenize");
		let parsed: Wast = wast::parser::parse(&lexed).unwrap();

		let mut data = Vec::new();

		Self::write_runtime(&mut data)?;

		for variant in parsed.directives {
			Self::write_variant(variant, &mut data)?;
		}

		Ok(data)
	}

	fn test(name: &str, source: &str) -> Result<()> {
		let data = Self::run_generation(source)?;
		let temp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
			.join(name)
			.with_extension("wast.lua");

		std::fs::write(&temp, &data)?;
		Self::run_command(&temp)
	}
}
