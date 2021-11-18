use super::level_2::list_to_range;
use crate::{
	backend::helper::{edition::Edition, register::Register, writer::Writer},
	data::{Arity, Code, Module},
};
use parity_wasm::elements::{BrTableData, Instruction};
use std::{fmt::Display, io::Result};

#[derive(PartialEq)]
pub enum Label {
	Block,
	If,
	Loop,
}

pub struct Body<'a> {
	spec: &'a dyn Edition,
	label_list: Vec<Label>,
	pub table_list: Vec<BrTableData>,
	pub reg: Register,
}

impl<'a> Body<'a> {
	pub fn new(spec: &'a dyn Edition) -> Self {
		Self {
			spec,
			label_list: vec![],
			table_list: Vec::new(),
			reg: Register::new(),
		}
	}

	pub fn gen(&mut self, index: usize, module: &Module) -> Result<Vec<u8>> {
		let mut w = Vec::new();

		module.code[index]
			.inst_list
			.iter()
			.try_for_each(|v| self.gen_inst(index, v, module, &mut w))?;

		Ok(w)
	}

	fn gen_jump(&mut self, up: u32, w: Writer) -> Result<()> {
		let up = up as usize;
		let level = self.label_list.len() - 1;
		let is_loop = self.label_list[level - up] == Label::Loop;

		self.spec.br_to_level(level, up, is_loop, w)
	}

	fn gen_br_if(&mut self, i: u32, f: &Code, w: Writer) -> Result<()> {
		let cond = f.var_name_of(self.reg.pop(1));

		write!(w, "if {} ~= 0 then ", cond)?;
		self.gen_jump(i, w)?;
		write!(w, "end ")
	}

	fn gen_br_table(&mut self, data: &BrTableData, f: &Code, w: Writer) -> Result<()> {
		let reg = f.var_name_of(self.reg.pop(1));

		for (r, t) in list_to_range(&data.table) {
			if r.len() == 1 {
				write!(w, "if {} == {} then ", reg, r.start)?;
			} else {
				write!(w, "if {0} >= {1} and {0} <= {2} then ", reg, r.start, r.end)?;
			}

			self.gen_jump(t, w)?;
			write!(w, "else")?;
		}

		write!(w, " ")?;
		self.gen_jump(data.default, w)?;
		write!(w, "end ")
	}

	fn gen_load(&mut self, t: &str, o: u32, f: &Code, w: Writer) -> Result<()> {
		let reg = f.var_name_of(self.reg.pop(1));

		self.reg.push(1);

		write!(w, "{0} = load.{1}(MEMORY_LIST[0], {0} + {2}) ", reg, t, o)
	}

	fn gen_store(&mut self, t: &str, o: u32, f: &Code, w: Writer) -> Result<()> {
		let val = f.var_name_of(self.reg.pop(1));
		let reg = f.var_name_of(self.reg.pop(1));

		write!(w, "store.{}(MEMORY_LIST[0], {} + {}, {}) ", t, reg, o, val)
	}

	fn gen_const<T: Display>(&mut self, val: T, f: &Code, w: Writer) -> Result<()> {
		let reg = f.var_name_of(self.reg.push(1));

		write!(w, "{} = {} ", reg, val)
	}

	fn gen_compare(&mut self, op: &str, f: &Code, w: Writer) -> Result<()> {
		let rhs = f.var_name_of(self.reg.pop(1));
		let lhs = f.var_name_of(self.reg.pop(1));

		self.reg.push(1);

		write!(w, "{1} = {1} {0} {2} and 1 or 0 ", op, lhs, rhs)
	}

	fn gen_unop_ex(&mut self, op: &str, f: &Code, w: Writer) -> Result<()> {
		let reg = f.var_name_of(self.reg.pop(1));

		self.reg.push(1);

		write!(w, "{1} = {0}({1}) ", op, reg)
	}

	fn gen_binop(&mut self, op: &str, f: &Code, w: Writer) -> Result<()> {
		let rhs = f.var_name_of(self.reg.pop(1));
		let lhs = f.var_name_of(self.reg.pop(1));

		self.reg.push(1);

		write!(w, "{1} = {1} {0} {2} ", op, lhs, rhs)
	}

	fn gen_binop_ex(&mut self, op: &str, f: &Code, w: Writer) -> Result<()> {
		let rhs = f.var_name_of(self.reg.pop(1));
		let lhs = f.var_name_of(self.reg.pop(1));

		self.reg.push(1);

		write!(w, "{1} = {0}({1}, {2}) ", op, lhs, rhs)
	}

	fn gen_call(&mut self, name: &str, f: &Code, a: &Arity, w: Writer) -> Result<()> {
		let bottom = self.reg.pop(a.num_param);

		self.reg.push(a.num_result);

		if a.num_result != 0 {
			let result = f.var_range_of(bottom, a.num_result).join(", ");

			write!(w, "{} =", result)?;
		}

		if a.num_param == 0 {
			write!(w, "{}()", name)
		} else {
			let param = f.var_range_of(bottom, a.num_param).join(", ");

			write!(w, "{}({})", name, param)
		}
	}

	fn gen_return(&mut self, num: u32, f: &Code, w: Writer) -> Result<()> {
		let top = self.reg.inner;
		let list = f.var_range_of(top - num, num).join(", ");

		self.reg.pop(num); // technically a no-op

		write!(w, "do return {} end ", list)
	}

	fn gen_inst(&mut self, index: usize, i: &Instruction, m: &Module, w: Writer) -> Result<()> {
		let func = &m.code[index];

		match i {
			Instruction::Unreachable => write!(w, "error('unreachable code entered')"),
			Instruction::Nop => {
				// no code
				Ok(())
			}
			Instruction::Block(_) => {
				self.reg.save();
				self.label_list.push(Label::Block);

				self.spec.start_block(w)
			}
			Instruction::Loop(_) => {
				self.reg.save();
				self.label_list.push(Label::Loop);

				self.spec.start_loop(self.label_list.len() - 1, w)
			}
			Instruction::If(_) => {
				let cond = func.var_name_of(self.reg.pop(1));

				self.reg.save();
				self.label_list.push(Label::If);

				self.spec.start_if(&cond, w)
			}
			Instruction::Else => {
				self.reg.load();
				self.reg.save();

				write!(w, "else ")
			}
			Instruction::End => {
				let rem = self.label_list.len().saturating_sub(1);

				match self.label_list.pop() {
					Some(Label::Block) => self.spec.end_block(rem, w)?,
					Some(Label::If) => self.spec.end_if(rem, w)?,
					Some(Label::Loop) => self.spec.end_loop(w)?,
					None => {
						let num = m.in_arity[index].num_result;

						if num != 0 {
							self.gen_return(num, func, w)?;
						}

						write!(w, "end ")?;
					}
				}

				self.reg.load();

				match self.label_list.last() {
					Some(Label::Block | Label::If) => self.spec.br_target(rem, false, w),
					Some(Label::Loop) => self.spec.br_target(rem, true, w),
					None => Ok(()),
				}
			}
			Instruction::Br(i) => self.gen_jump(*i, w),
			Instruction::BrIf(i) => self.gen_br_if(*i, func, w),
			Instruction::BrTable(data) => self.gen_br_table(data, func, w),
			Instruction::Return => {
				let num = m.in_arity[index].num_result;

				self.gen_return(num, func, w)
			}
			Instruction::Call(i) => {
				let name = format!("FUNC_LIST[{}]", i);
				let arity = m.arity_of(*i as usize);

				self.gen_call(&name, func, arity, w)
			}
			Instruction::CallIndirect(i, t) => {
				let index = func.var_name_of(self.reg.pop(1));
				let name = format!("TABLE_LIST[{}].data[{}]", t, index);
				let types = m.parent.type_section().unwrap().types();
				let arity = Arity::from_index(types, *i);

				self.gen_call(&name, func, &arity, w)
			}
			Instruction::Drop => {
				self.reg.pop(1);

				Ok(())
			}
			Instruction::Select => {
				let cond = func.var_name_of(self.reg.pop(1));
				let v2 = func.var_name_of(self.reg.pop(1));
				let v1 = func.var_name_of(self.reg.pop(1));

				self.reg.push(1);

				write!(w, "if {} == 0 then ", cond)?;
				write!(w, "{} = {} ", v1, v2)?;
				write!(w, "end ")
			}
			Instruction::GetLocal(i) => {
				let reg = func.var_name_of(self.reg.push(1));
				let var = func.var_name_of(*i);

				write!(w, "{} = {} ", reg, var)
			}
			Instruction::SetLocal(i) => {
				let var = func.var_name_of(*i);
				let reg = func.var_name_of(self.reg.pop(1));

				write!(w, "{} = {} ", var, reg)
			}
			Instruction::TeeLocal(i) => {
				let var = func.var_name_of(*i);
				let reg = func.var_name_of(self.reg.pop(1));

				self.reg.push(1);

				write!(w, "{} = {} ", var, reg)
			}
			Instruction::GetGlobal(i) => {
				let reg = func.var_name_of(self.reg.push(1));

				write!(w, "{} = GLOBAL_LIST[{}].value ", reg, i)
			}
			Instruction::SetGlobal(i) => {
				let reg = func.var_name_of(self.reg.pop(1));

				write!(w, "GLOBAL_LIST[{}].value = {} ", i, reg)
			}
			Instruction::I32Load(_, o) => self.gen_load("i32", *o, func, w),
			Instruction::I64Load(_, o) => self.gen_load("i64", *o, func, w),
			Instruction::F32Load(_, o) => self.gen_load("f32", *o, func, w),
			Instruction::F64Load(_, o) => self.gen_load("f64", *o, func, w),
			Instruction::I32Load8S(_, o) => self.gen_load("i32_i8", *o, func, w),
			Instruction::I32Load8U(_, o) => self.gen_load("i32_u8", *o, func, w),
			Instruction::I32Load16S(_, o) => self.gen_load("i32_i16", *o, func, w),
			Instruction::I32Load16U(_, o) => self.gen_load("i32_u16", *o, func, w),
			Instruction::I64Load8S(_, o) => self.gen_load("i64_i8", *o, func, w),
			Instruction::I64Load8U(_, o) => self.gen_load("i64_u8", *o, func, w),
			Instruction::I64Load16S(_, o) => self.gen_load("i64_i16", *o, func, w),
			Instruction::I64Load16U(_, o) => self.gen_load("i64_u16", *o, func, w),
			Instruction::I64Load32S(_, o) => self.gen_load("i64_i32", *o, func, w),
			Instruction::I64Load32U(_, o) => self.gen_load("i64_u32", *o, func, w),
			Instruction::I32Store(_, o) => self.gen_store("i32", *o, func, w),
			Instruction::I64Store(_, o) => self.gen_store("i64", *o, func, w),
			Instruction::F32Store(_, o) => self.gen_store("f32", *o, func, w),
			Instruction::F64Store(_, o) => self.gen_store("f64", *o, func, w),
			Instruction::I32Store8(_, o) => self.gen_store("i32_n8", *o, func, w),
			Instruction::I32Store16(_, o) => self.gen_store("i32_n16", *o, func, w),
			Instruction::I64Store8(_, o) => self.gen_store("i64_n8", *o, func, w),
			Instruction::I64Store16(_, o) => self.gen_store("i64_n16", *o, func, w),
			Instruction::I64Store32(_, o) => self.gen_store("i64_n32", *o, func, w),
			Instruction::CurrentMemory(index) => {
				let reg = func.var_name_of(self.reg.push(1));

				write!(w, "{} = rt.memory.size(MEMORY_LIST[{}])", reg, index)
			}
			Instruction::GrowMemory(index) => {
				let reg = func.var_name_of(self.reg.pop(1));

				self.reg.push(1);

				write!(w, "{0} = rt.memory.grow(MEMORY_LIST[{1}], {0})", reg, index)
			}
			Instruction::I32Const(v) => self.gen_const(v, func, w),
			Instruction::I64Const(v) => self.gen_const(self.spec.i64(*v), func, w),
			Instruction::F32Const(v) => self.gen_const(f32::from_bits(*v), func, w),
			Instruction::F64Const(v) => self.gen_const(f64::from_bits(*v), func, w),
			Instruction::I32Eqz | Instruction::I64Eqz => {
				let reg = func.var_name_of(self.reg.pop(1));

				self.reg.push(1);

				write!(w, "{} = {} == 0 and 1 or 0 ", reg, reg)
			}
			Instruction::I32Eq | Instruction::I64Eq | Instruction::F32Eq | Instruction::F64Eq => {
				self.gen_compare("==", func, w)
			}
			Instruction::I32Ne | Instruction::I64Ne | Instruction::F32Ne | Instruction::F64Ne => {
				self.gen_compare("~=", func, w)
			}
			// note that signed comparisons of all types behave the same so
			// they can be condensed using Lua's operators
			Instruction::I32LtU => self.gen_binop_ex("lt.u32", func, w),
			Instruction::I32LtS | Instruction::I64LtS | Instruction::F32Lt | Instruction::F64Lt => {
				self.gen_compare("<", func, w)
			}
			Instruction::I32GtU => self.gen_binop_ex("gt.u32", func, w),
			Instruction::I32GtS | Instruction::I64GtS | Instruction::F32Gt | Instruction::F64Gt => {
				self.gen_compare(">", func, w)
			}
			Instruction::I32LeU => self.gen_binop_ex("le.u32", func, w),
			Instruction::I32LeS | Instruction::I64LeS | Instruction::F32Le | Instruction::F64Le => {
				self.gen_compare("<=", func, w)
			}
			Instruction::I32GeU => self.gen_binop_ex("ge.u32", func, w),
			Instruction::I32GeS | Instruction::I64GeS | Instruction::F32Ge | Instruction::F64Ge => {
				self.gen_compare(">=", func, w)
			}
			Instruction::I64LtU => self.gen_binop_ex("lt.u64", func, w),
			Instruction::I64GtU => self.gen_binop_ex("gt.u64", func, w),
			Instruction::I64LeU => self.gen_binop_ex("le.u64", func, w),
			Instruction::I64GeU => self.gen_binop_ex("ge.u64", func, w),
			Instruction::I32Clz => self.gen_unop_ex("clz.i32", func, w),
			Instruction::I32Ctz => self.gen_unop_ex("ctz.i32", func, w),
			Instruction::I32Popcnt => self.gen_unop_ex("popcnt.i32", func, w),
			Instruction::I32DivS => self.gen_binop_ex("div.i32", func, w),
			Instruction::I32DivU => self.gen_binop_ex("div.u32", func, w),
			Instruction::I32RemS => self.gen_binop_ex("rem.i32", func, w),
			Instruction::I32RemU => self.gen_binop_ex("rem.u32", func, w),
			Instruction::I32And => self.gen_binop_ex("band.i32", func, w),
			Instruction::I32Or => self.gen_binop_ex("bor.i32", func, w),
			Instruction::I32Xor => self.gen_binop_ex("bxor.i32", func, w),
			Instruction::I32Shl => self.gen_binop_ex("shl.i32", func, w),
			Instruction::I32ShrS => self.gen_binop_ex("shr.i32", func, w),
			Instruction::I32ShrU => self.gen_binop_ex("shr.u32", func, w),
			Instruction::I32Rotl => self.gen_binop_ex("rotl.i32", func, w),
			Instruction::I32Rotr => self.gen_binop_ex("rotr.i32", func, w),
			Instruction::I64Clz => self.gen_unop_ex("clz.i64", func, w),
			Instruction::I64Ctz => self.gen_unop_ex("ctz.i64", func, w),
			Instruction::I64Popcnt => self.gen_unop_ex("popcnt.i64", func, w),
			Instruction::I64DivS => self.gen_binop_ex("div.i64", func, w),
			Instruction::I64DivU => self.gen_binop_ex("div.u64", func, w),
			Instruction::I64RemS => self.gen_binop_ex("rem.i64", func, w),
			Instruction::I64RemU => self.gen_binop_ex("rem.u64", func, w),
			Instruction::I64And => self.gen_binop_ex("band.i64", func, w),
			Instruction::I64Or => self.gen_binop_ex("bor.i64", func, w),
			Instruction::I64Xor => self.gen_binop_ex("bxor.i64", func, w),
			Instruction::I64Shl => self.gen_binop_ex("shl.i64", func, w),
			Instruction::I64ShrS => self.gen_binop_ex("shr.i64", func, w),
			Instruction::I64ShrU => self.gen_binop_ex("shr.u64", func, w),
			Instruction::I64Rotl => self.gen_binop_ex("rotl.i64", func, w),
			Instruction::I64Rotr => self.gen_binop_ex("rotr.i64", func, w),
			Instruction::F32Abs | Instruction::F64Abs => self.gen_unop_ex("math.abs", func, w),
			Instruction::F32Neg | Instruction::F64Neg => {
				let reg = func.var_name_of(self.reg.pop(1));

				self.reg.push(1);

				write!(w, "{} = -{} ", reg, reg)
			}
			Instruction::F32Ceil | Instruction::F64Ceil => self.gen_unop_ex("math.ceil", func, w),
			Instruction::F32Floor | Instruction::F64Floor => {
				self.gen_unop_ex("math.floor", func, w)
			}
			Instruction::F32Trunc | Instruction::F64Trunc => self.gen_unop_ex("trunc.f", func, w),
			Instruction::F32Nearest | Instruction::F64Nearest => {
				self.gen_unop_ex("nearest.f", func, w)
			}
			Instruction::F32Sqrt | Instruction::F64Sqrt => self.gen_unop_ex("math.sqrt", func, w),
			Instruction::I32Add
			| Instruction::I64Add
			| Instruction::F32Add
			| Instruction::F64Add => self.gen_binop("+", func, w),
			Instruction::I32Sub
			| Instruction::I64Sub
			| Instruction::F32Sub
			| Instruction::F64Sub => self.gen_binop("-", func, w),
			Instruction::I32Mul
			| Instruction::I64Mul
			| Instruction::F32Mul
			| Instruction::F64Mul => self.gen_binop("*", func, w),
			Instruction::F32Div | Instruction::F64Div => self.gen_binop("/", func, w),
			Instruction::F32Min | Instruction::F64Min => self.gen_binop_ex("math.min", func, w),
			Instruction::F32Max | Instruction::F64Max => self.gen_binop_ex("math.max", func, w),
			Instruction::F32Copysign | Instruction::F64Copysign => {
				self.gen_unop_ex("math.sign", func, w)
			}
			Instruction::I32WrapI64 => self.gen_unop_ex("wrap.i64_i32", func, w),
			Instruction::I32TruncSF32 => self.gen_unop_ex("trunc.f32_i32", func, w),
			Instruction::I32TruncUF32 => self.gen_unop_ex("trunc.f32_u32", func, w),
			Instruction::I32TruncSF64 => self.gen_unop_ex("trunc.f64_i32", func, w),
			Instruction::I32TruncUF64 => self.gen_unop_ex("trunc.f64_u32", func, w),
			Instruction::I64ExtendSI32 => self.gen_unop_ex("extend.i32_i64", func, w),
			Instruction::I64ExtendUI32 => self.gen_unop_ex("extend.i32_u64", func, w),
			Instruction::I64TruncSF32 => self.gen_unop_ex("trunc.f32_i64", func, w),
			Instruction::I64TruncUF32 => self.gen_unop_ex("trunc.f32_u64", func, w),
			Instruction::I64TruncSF64 => self.gen_unop_ex("trunc.f64_i64", func, w),
			Instruction::I64TruncUF64 => self.gen_unop_ex("trunc.f64_u64", func, w),
			Instruction::F32ConvertSI32 => self.gen_unop_ex("convert.i32_f32", func, w),
			Instruction::F32ConvertUI32 => self.gen_unop_ex("convert.u32_f32", func, w),
			Instruction::F32ConvertSI64 => self.gen_unop_ex("convert.i64_f32", func, w),
			Instruction::F32ConvertUI64 => self.gen_unop_ex("convert.u64_f32", func, w),
			Instruction::F32DemoteF64 => self.gen_unop_ex("demote.f64_f32", func, w),
			Instruction::F64ConvertSI32 => self.gen_unop_ex("convert.f64_i32", func, w),
			Instruction::F64ConvertUI32 => self.gen_unop_ex("convert.f64_u32", func, w),
			Instruction::F64ConvertSI64 => self.gen_unop_ex("convert.f64_i64", func, w),
			Instruction::F64ConvertUI64 => self.gen_unop_ex("convert.f64_u64", func, w),
			Instruction::F64PromoteF32 => self.gen_unop_ex("promote.f32_f64", func, w),
			Instruction::I32ReinterpretF32 => self.gen_unop_ex("reinterpret.f32_i32", func, w),
			Instruction::I64ReinterpretF64 => self.gen_unop_ex("reinterpret.f64_i64", func, w),
			Instruction::F32ReinterpretI32 => self.gen_unop_ex("reinterpret.i32_f32", func, w),
			Instruction::F64ReinterpretI64 => self.gen_unop_ex("reinterpret.i64_f64", func, w),
		}
	}
}
