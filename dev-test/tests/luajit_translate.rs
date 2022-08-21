use std::{
	io::{Result, Write},
	path::PathBuf,
};

use wasm_ast::module::{Module, TypeInfo};
use wast::{
	core::{Expression, Instruction},
	AssertExpression, WastExecute, WastInvoke, Wat,
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

	fn write_assert_trap(data: &mut WastExecute, w: &mut dyn Write) -> Result<()> {
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
			WastExecute::Wat(data) => {
				let bytes = match data {
					Wat::Module(ast) => ast.encode().unwrap(),
					Wat::Component(_) => unimplemented!(),
				};
				let data = Module::from_data(&bytes);

				writeln!(w, "assert_trap((function()")?;
				codegen_luajit::from_module_untyped(&data, w)?;
				writeln!(w, "end)(), linked)")
			}
		}
	}

	fn write_assert_return(
		data: &mut WastExecute,
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
		write!(w, "{runtime}")?;
		writeln!(w, "end)()")?;

		writeln!(w, "{ASSERTION}")
	}

	fn write_module(data: &Module, name: Option<&str>, w: &mut dyn Write) -> Result<()> {
		let type_info = TypeInfo::from_module(data);

		writeln!(w, r#"loaded["temp"] = (function()"#)?;
		codegen_luajit::from_module_typed(data, &type_info, w)?;
		writeln!(w, "end)()(linked)")?;

		if let Some(name) = name {
			writeln!(w, r#"loaded["{name}"] = loaded["temp"]"#)?;
		}

		Ok(())
	}
}

static DO_NOT_RUN: [&str; 58] = [
	"names.wast",
	"skip-stack-guard-page.wast",
	"simd_address.wast",
	"simd_align.wast",
	"simd_bit_shift.wast",
	"simd_bitwise.wast",
	"simd_boolean.wast",
	"simd_const.wast",
	"simd_conversions.wast",
	"simd_f32x4_arith.wast",
	"simd_f32x4_cmp.wast",
	"simd_f32x4_pmin_pmax.wast",
	"simd_f32x4_rounding.wast",
	"simd_f32x4.wast",
	"simd_f64x2_arith.wast",
	"simd_f64x2_cmp.wast",
	"simd_f64x2_pmin_pmax.wast",
	"simd_f64x2_rounding.wast",
	"simd_f64x2.wast",
	"simd_i16x8_arith.wast",
	"simd_i16x8_arith2.wast",
	"simd_i16x8_cmp.wast",
	"simd_i16x8_extadd_pairwise_i8x16.wast",
	"simd_i16x8_extmul_i8x16.wast",
	"simd_i16x8_q15mulr_sat_s.wast",
	"simd_i16x8_sat_arith.wast",
	"simd_i32x4_arith.wast",
	"simd_i32x4_arith2.wast",
	"simd_i32x4_cmp.wast",
	"simd_i32x4_dot_i16x8.wast",
	"simd_i32x4_extadd_pairwise_i16x8.wast",
	"simd_i32x4_extmul_i16x8.wast",
	"simd_i32x4_trunc_sat_f32x4.wast",
	"simd_i32x4_trunc_sat_f64x2.wast",
	"simd_i64x2_arith.wast",
	"simd_i64x2_arith2.wast",
	"simd_i64x2_cmp.wast",
	"simd_i64x2_extmul_i32x4.wast",
	"simd_i8x16_arith.wast",
	"simd_i8x16_arith2.wast",
	"simd_i8x16_cmp.wast",
	"simd_i8x16_sat_arith.wast",
	"simd_int_to_int_extend.wast",
	"simd_lane.wast",
	"simd_load_extend.wast",
	"simd_load_splat.wast",
	"simd_load_zero.wast",
	"simd_load.wast",
	"simd_load16_lane.wast",
	"simd_load32_lane.wast",
	"simd_load64_lane.wast",
	"simd_load8_lane.wast",
	"simd_splat.wast",
	"simd_store.wast",
	"simd_store16_lane.wast",
	"simd_store32_lane.wast",
	"simd_store64_lane.wast",
	"simd_store8_lane.wast",
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
