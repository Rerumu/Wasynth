use std::{collections::BTreeSet, io::Result};

use parity_wasm::elements::Instruction;

use crate::{
	backend::{
		edition::data::Edition,
		helper::writer::{write_ordered, Writer},
		visitor::{localize, memory, register},
	},
	data::Module,
};

use super::level_1::Body;

pub fn gen_init_expression(code: &[Instruction], w: Writer) -> Result<()> {
	assert!(code.len() == 2);

	let inst = code.first().unwrap();

	match *inst {
		Instruction::I32Const(v) => write!(w, "{} ", v),
		Instruction::I64Const(v) => write!(w, "{} ", v),
		Instruction::F32Const(v) => write!(w, "{} ", f32::from_bits(v)),
		Instruction::F64Const(v) => write!(w, "{} ", f64::from_bits(v)),
		Instruction::GetGlobal(i) => write!(w, "GLOBAL_LIST[{}].value ", i),
		_ => unreachable!(),
	}
}

fn gen_prelude(num_stack: u32, num_param: u32, num_local: u32, w: Writer) -> Result<()> {
	let num_reg = num_stack - num_param - num_local;

	write!(w, "function(")?;
	write_ordered("param", num_param, w)?;
	write!(w, ")")?;

	if num_local != 0 {
		let zero = vec!["0"; num_local as usize].join(", ");

		write!(w, "local ")?;
		write_ordered("var", num_local, w)?;
		write!(w, "= {} ", zero)?;
	}

	if num_reg != 0 {
		write!(w, "local ")?;
		write_ordered("reg", num_reg, w)?;
		write!(w, " ")?;
	}

	Ok(())
}

fn gen_memory(set: BTreeSet<u8>, w: Writer) -> Result<()> {
	set.into_iter()
		.try_for_each(|i| write!(w, "local memory_at_{0} = MEMORY_LIST[{0}]", i))
}

fn gen_localize(set: BTreeSet<(&str, &str)>, w: Writer) -> Result<()> {
	set.into_iter()
		.try_for_each(|v| write!(w, "local {0}_{1} = {0}.{1} ", v.0, v.1))
}

pub fn gen_function(spec: &dyn Edition, index: usize, m: &Module, w: Writer) -> Result<()> {
	let mem_set = memory::visit(m, index);
	let loc_set = localize::visit(m, index);
	let num_stack = register::visit(m, index);

	let num_param = m.in_arity[index].num_param;
	let num_local = m.code[index].num_local;

	let mut inner = Body::new(spec, num_param + num_local);

	gen_prelude(num_stack, num_param, num_local, w)?;
	gen_memory(mem_set, w)?;
	gen_localize(loc_set, w)?;
	inner.generate(index, m, w)?;

	assert!(
		inner.reg.last == num_stack,
		"Mismatched register allocation"
	);

	Ok(())
}
