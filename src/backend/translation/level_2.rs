use super::level_1::Body;
use crate::{
	backend::helper::{
		edition::Edition,
		writer::{write_ordered, Writer},
	},
	data::Module,
};
use parity_wasm::elements::Instruction;
use std::{
	io::{Result, Write},
	ops::Range,
};

pub fn list_to_range(list: &[u32]) -> Vec<(Range<usize>, u32)> {
	let mut result = Vec::new();
	let mut index = 0;

	while index < list.len() {
		let start = index;

		loop {
			index += 1;

			// if end of list or next value is not equal, break
			if index == list.len() || list[index - 1] != list[index] {
				break;
			}
		}

		result.push((start..index, list[start]));
	}

	result
}

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

fn gen_prelude(num_param: u32, num_local: u32) -> Result<Vec<u8>> {
	let mut w = Vec::new();

	write!(w, "function(")?;
	write_ordered("param", num_param, &mut w)?;
	write!(w, ")")?;

	if num_local != 0 {
		let zero = vec!["0"; num_local as usize].join(", ");

		write!(w, "local ")?;
		write_ordered("var", num_local, &mut w)?;
		write!(w, "= {} ", zero)?;
	}

	Ok(w)
}

fn gen_reg_list(last: u32, num_param: u32, num_local: u32) -> Result<Vec<u8>> {
	let mut w = Vec::new();
	let num = last - num_local - num_param;

	if num != 0 {
		write!(w, "local ")?;
		write_ordered("reg", num, &mut w)?;
		write!(w, " ")?;
	}

	Ok(w)
}

pub fn gen_function(spec: &dyn Edition, index: usize, m: &Module, w: Writer) -> Result<()> {
	let mut inner = Body::new(spec);
	let num_param = m.in_arity[index].num_param;
	let num_local = m.code[index].num_local;

	inner.reg.push(num_param + num_local);

	let prelude = gen_prelude(num_param, num_local)?;
	let body = inner.gen(index, m)?;
	let reg = gen_reg_list(inner.reg.last, num_param, num_local)?;

	w.write_all(&prelude)?;
	w.write_all(&reg)?;
	w.write_all(&body)
}
