#![cfg(test)]

use std::{
	io::{Result as IResult, Write},
	marker::PhantomData,
	path::{Path, PathBuf},
	process::Command,
};

use parity_wasm::elements::Module as BinModule;
use wast::{
	core::{Expression, Instruction, Module as AstModule},
	parser::ParseBuffer,
	token::{Float32, Float64, Id},
	AssertExpression, NanPattern, QuoteWat, Wast, WastDirective, WastExecute, WastInvoke, Wat,
};

use wasm_ast::builder::TypeInfo;

static ASSERTION: &str = include_str!("assertion.lua");

macro_rules! write_assert_number {
	($name:ident, $generic:ty, $reader:ty) => {
		fn $name(data: &NanPattern<$generic>, w: &mut dyn Write) -> IResult<()> {
			match data {
				NanPattern::CanonicalNan => write!(w, "LUA_NAN_CANONICAL"),
				NanPattern::ArithmeticNan => write!(w, "LUA_NAN_ARITHMETIC"),
				NanPattern::Value(data) => {
					let number = <$reader>::from_bits(data.bits);
					let sign = if number.is_sign_negative() { "-" } else { "" };

					if number.is_infinite() {
						write!(w, "{sign}LUA_INFINITY")
					} else if number.is_nan() {
						write!(w, "{sign}LUA_NAN_DEFAULT")
					} else {
						write!(w, "{number}")
					}
				}
			}
		}
	};
}

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

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> IResult<()>;

	fn write_assert_trap(data: &WastExecute, w: &mut dyn Write) -> IResult<()>;

	fn write_assert_return(
		data: &WastExecute,
		result: &[AssertExpression],
		w: &mut dyn Write,
	) -> IResult<()>;

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> IResult<()>;

	fn write_runtime(w: &mut dyn Write) -> IResult<()>;

	fn write_module(typed: &TypedModule, w: &mut dyn Write) -> IResult<()>;
}

struct LuaJIT;

impl LuaJIT {
	fn write_expression(data: &Expression, w: &mut dyn Write) -> IResult<()> {
		let data = &data.instrs;

		assert_eq!(data.len(), 1, "Only one instruction supported");

		match &data[0] {
			Instruction::I32Const(v) => write!(w, "{v}"),
			Instruction::I64Const(v) => write!(w, "{v}LL"),
			Instruction::F32Const(v) => write!(w, "{}", f32::from_bits(v.bits)),
			Instruction::F64Const(v) => write!(w, "{}", f64::from_bits(v.bits)),
			_ => panic!("Unsupported instruction"),
		}
	}

	write_assert_number!(write_assert_maybe_f32, Float32, f32);
	write_assert_number!(write_assert_maybe_f64, Float64, f64);

	fn write_simple_expression(data: &AssertExpression, w: &mut dyn Write) -> IResult<()> {
		match data {
			AssertExpression::I32(v) => write!(w, "{v}"),
			AssertExpression::I64(v) => write!(w, "{v}LL"),
			AssertExpression::F32(v) => Self::write_assert_maybe_f32(v, w),
			AssertExpression::F64(v) => Self::write_assert_maybe_f64(v, w),
			_ => panic!("Unsupported expression"),
		}
	}

	fn write_call_of(handler: &str, data: &WastInvoke, w: &mut dyn Write) -> IResult<()> {
		let name = TypedModule::resolve_id(data.module);
		let func = data.name;

		write!(w, "{handler}(")?;
		write!(w, r#"loaded["{name}"].func_list["{func}"]"#)?;

		data.args.iter().try_for_each(|v| {
			write!(w, ", ")?;
			Self::write_expression(v, w)
		})?;

		write!(w, ")")
	}
}

impl Target for LuaJIT {
	fn executable() -> String {
		std::env::var("LUAJIT_PATH").unwrap_or_else(|_| "luajit".to_string())
	}

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> IResult<()> {
		writeln!(w, r#"linked["{post}"] = loaded["{pre}"]"#)
	}

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> IResult<()> {
		Self::write_call_of("raw_invoke", data, w)?;
		writeln!(w)
	}

	fn write_assert_trap(data: &WastExecute, w: &mut dyn Write) -> IResult<()> {
		match data {
			WastExecute::Invoke(data) => {
				Self::write_call_of("assert_trap", data, w)?;
				writeln!(w)
			}
			WastExecute::Get { module, global } => {
				let name = TypedModule::resolve_id(*module);

				write!(w, "assert_neq(")?;
				write!(w, r#"loaded["{name}"].global_list["{global}"].value"#)?;
				writeln!(w, ", nil)")
			}
			WastExecute::Wat(_) => panic!("Wat not supported"),
		}
	}

	fn write_assert_return(
		data: &WastExecute,
		result: &[AssertExpression],
		w: &mut dyn Write,
	) -> IResult<()> {
		match data {
			WastExecute::Invoke(data) => {
				write!(w, "assert_return(")?;
				write!(w, "{{")?;
				Self::write_call_of("raw_invoke", data, w)?;
				write!(w, "}}, {{")?;

				for v in result {
					Self::write_simple_expression(v, w)?;
					write!(w, ", ")?;
				}

				writeln!(w, "}})")
			}
			WastExecute::Get { module, global } => {
				let name = TypedModule::resolve_id(*module);

				write!(w, "assert_eq(")?;
				write!(w, r#"loaded["{name}"].global_list["{global}"].value"#)?;
				write!(w, ", ")?;
				Self::write_simple_expression(&result[0], w)?;
				writeln!(w, ")")
			}
			WastExecute::Wat(_) => panic!("Wat not supported"),
		}
	}

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> IResult<()> {
		Self::write_call_of("assert_exhaustion", data, w)?;
		writeln!(w)
	}

	fn write_runtime(w: &mut dyn Write) -> IResult<()> {
		let runtime = codegen_luajit::RUNTIME;

		writeln!(w, "{ASSERTION}")?;
		writeln!(w, "local rt = (function() {runtime} end)()")
	}

	fn write_module(typed: &TypedModule, w: &mut dyn Write) -> IResult<()> {
		write!(w, r#"loaded["{}"] = (function() "#, typed.name)?;
		codegen_luajit::from_module_typed(typed.module, &typed.type_info, w)?;
		writeln!(w, "end)()(nil)")
	}
}

struct Luau;

impl Target for Luau {
	fn executable() -> String {
		std::env::var("LUAU_PATH").unwrap_or_else(|_| "luajit".to_string())
	}

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> IResult<()> {
		writeln!(w, r#"linked["{post}"] = loaded["{pre}"]"#)
	}

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> IResult<()> {
		todo!();
	}

	fn write_assert_trap(data: &WastExecute, w: &mut dyn Write) -> IResult<()> {
		todo!();
	}

	fn write_assert_return(
		data: &WastExecute,
		result: &[AssertExpression],
		w: &mut dyn Write,
	) -> IResult<()> {
		todo!();
	}

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> IResult<()> {
		todo!();
	}

	fn write_runtime(w: &mut dyn Write) -> IResult<()> {
		let runtime = codegen_luau::RUNTIME;

		writeln!(w, "{ASSERTION}")?;
		writeln!(w, "local rt = (function() {runtime} end)()")
	}

	fn write_module(typed: &TypedModule, w: &mut dyn Write) -> IResult<()> {
		write!(w, r#"loaded["{}"] = (function() "#, typed.name)?;
		codegen_luau::from_module_typed(typed.module, &typed.type_info, w)?;
		writeln!(w, "end)()(nil)")
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

struct Tester<T> {
	_marker: PhantomData<T>,
}

impl<T: Target> Tester<T> {
	fn test(name: &str, source: &str) -> IResult<()> {
		if let Some(data) = Self::run_generation(source)? {
			let temp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
				.join(name)
				.with_extension("wast.lua");

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
				let typed = TypedModule::from_id(ast.id, &temp);

				T::write_module(&typed, w)?;
			}
			WastDirective::Register { name, module, .. } => {
				let pre = TypedModule::resolve_id(module);

				T::write_register(name, pre, w)?;
			}
			WastDirective::Invoke(data) => {
				T::write_invoke(&data, w)?;
			}
			WastDirective::AssertTrap { exec, .. } => {
				T::write_assert_trap(&exec, w)?;
			}
			WastDirective::AssertReturn { exec, results, .. } => {
				T::write_assert_return(&exec, &results, w)?;
			}
			WastDirective::AssertExhaustion { call, .. } => {
				T::write_assert_exhaustion(&call, w)?;
			}
			_ => {}
		}

		Ok(())
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

static DO_NOT_RUN: [&str; 8] = [
	"binary-leb128.wast",
	"conversions.wast",
	"float_exprs.wast",
	"float_literals.wast",
	"float_memory.wast",
	"float_misc.wast",
	"names.wast",
	"skip-stack-guard-page.wast",
];

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
