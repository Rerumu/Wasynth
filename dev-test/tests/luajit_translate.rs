use std::{
	io::{Result, Write},
	path::PathBuf,
};

use wast::{
	core::{Expression, Instruction},
	token::{Float32, Float64},
	AssertExpression, WastExecute, WastInvoke,
};

use target::{Target, TypedModule};

mod target;

static ASSERTION: &str = include_str!("assertion.lua");

struct LuaJIT;

impl LuaJIT {
	fn write_expression(data: &Expression, w: &mut dyn Write) -> Result<()> {
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

	fn write_simple_expression(data: &AssertExpression, w: &mut dyn Write) -> Result<()> {
		match data {
			AssertExpression::I32(v) => write!(w, "{v}"),
			AssertExpression::I64(v) => write!(w, "{v}LL"),
			AssertExpression::F32(v) => Self::write_assert_maybe_f32(v, w),
			AssertExpression::F64(v) => Self::write_assert_maybe_f64(v, w),
			_ => panic!("Unsupported expression"),
		}
	}

	fn write_call_of(handler: &str, data: &WastInvoke, w: &mut dyn Write) -> Result<()> {
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

	fn write_register(post: &str, pre: &str, w: &mut dyn Write) -> Result<()> {
		writeln!(w, r#"linked["{post}"] = loaded["{pre}"]"#)
	}

	fn write_invoke(data: &WastInvoke, w: &mut dyn Write) -> Result<()> {
		Self::write_call_of("raw_invoke", data, w)?;
		writeln!(w)
	}

	fn write_assert_trap(data: &WastExecute, w: &mut dyn Write) -> Result<()> {
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
	) -> Result<()> {
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

	fn write_assert_exhaustion(data: &WastInvoke, w: &mut dyn Write) -> Result<()> {
		Self::write_call_of("assert_exhaustion", data, w)?;
		writeln!(w)
	}

	fn write_runtime(w: &mut dyn Write) -> Result<()> {
		let runtime = codegen_luajit::RUNTIME;

		writeln!(w, "{ASSERTION}")?;
		writeln!(w, "local rt = (function() {runtime} end)()")
	}

	fn write_module(typed: &TypedModule, w: &mut dyn Write) -> Result<()> {
		write!(w, r#"loaded["{}"] = (function() "#, typed.name())?;
		codegen_luajit::from_module_typed(typed.module(), typed.type_info(), w)?;
		writeln!(w, "end)()(nil)")
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

	LuaJIT::test(name, &source).unwrap();
}
