use std::{
	io::{Result, Write},
	path::PathBuf,
};

use parity_wasm::elements::Module;
use wasm_ast::builder::TypeInfo;
use wast::{
	core::{Expression, Instruction},
	AssertExpression, WastExecute, WastInvoke,
};

use target::{get_name_from_id, Target};

mod target;

static ASSERTION: &str = include_str!("luajit_assert.lua");

struct LuaJIT;

impl LuaJIT {
	fn write_expression(data: &Expression, w: &mut dyn Write) -> Result<()> {
		let data = &data.instrs;

		assert_eq!(data.len(), 1, "Only one instruction supported");

		match &data[0] {
			Instruction::I32Const(v) => write!(w, "{v}"),
			Instruction::I64Const(v) => write!(w, "{v}LL"),
			Instruction::F32Const(v) => target::write_f32(f32::from_bits(v.bits), w),
			Instruction::F64Const(v) => target::write_f64(f64::from_bits(v.bits), w),
			_ => panic!("Unsupported instruction"),
		}
	}

	fn write_simple_expression(data: &AssertExpression, w: &mut dyn Write) -> Result<()> {
		match data {
			AssertExpression::I32(v) => write!(w, "{v}"),
			AssertExpression::I64(v) => write!(w, "{v}LL"),
			AssertExpression::F32(v) => target::write_f32_nan(v, w),
			AssertExpression::F64(v) => target::write_f64_nan(v, w),
			_ => panic!("Unsupported expression"),
		}
	}

	fn write_call_of(handler: &str, data: &WastInvoke, w: &mut dyn Write) -> Result<()> {
		let name = get_name_from_id(data.module);
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
				let name = get_name_from_id(*module);

				write!(w, "assert_neq(")?;
				write!(w, r#"loaded["{name}"].global_list["{global}"].value"#)?;
				writeln!(w, ", nil)")
			}
			WastExecute::Wat(_) => {
				// FIXME: Assert the `start` function

				Ok(())
			}
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
				let name = get_name_from_id(*module);

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

		writeln!(w, "local rt = (function()")?;
		writeln!(w, "{runtime}")?;
		writeln!(w, "end)()")?;

		writeln!(w, "{ASSERTION}")
	}

	fn write_module(data: &Module, name: Option<&str>, w: &mut dyn Write) -> Result<()> {
		let type_info = TypeInfo::from_module(data);

		write!(w, r#"loaded["temp"] = (function() "#)?;
		codegen_luajit::from_module_typed(data, &type_info, w)?;
		writeln!(w, "end)()(linked)")?;

		if let Some(name) = name {
			writeln!(w, r#"loaded["{name}"] = loaded["temp"]"#)?;
		}

		Ok(())
	}
}

static DO_NOT_RUN: [&str; 4] = [
	"binary-leb128.wast",
	"conversions.wast",
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
