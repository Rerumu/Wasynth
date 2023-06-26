use std::collections::HashSet;

use crate::node::{
	Align, Expression, GetGlobal, LoadAt, Local, ResultList, SetTemporary, Statement, Temporary,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReadType {
	Local(usize),
	Global(usize),
	Memory(usize),
}

pub struct Slot {
	read: HashSet<ReadType>,
	data: Expression,
}

impl Slot {
	const fn is_temporary(&self, id: usize) -> bool {
		matches!(self.data, Expression::GetTemporary(ref v) if v.var() == id)
	}

	pub fn has_read(&self, id: ReadType) -> bool {
		self.read.contains(&id)
	}

	pub fn has_global_read(&self) -> bool {
		self.read.iter().any(|r| matches!(r, ReadType::Global(_)))
	}

	pub fn has_memory_read(&self) -> bool {
		self.read.iter().any(|r| matches!(r, ReadType::Memory(_)))
	}
}

#[derive(Default)]
pub struct Stack {
	var_list: Vec<Slot>,
	pub capacity: usize,
	pub previous: usize,
}

impl Stack {
	pub fn len(&self) -> usize {
		self.var_list.len()
	}

	pub fn split_last(&mut self, num_param: usize, num_result: usize) -> Self {
		let desired = self.len() - num_param;
		let var_list = self.var_list.split_off(desired);

		Self {
			var_list,
			capacity: self.capacity.max(desired + num_result),
			previous: self.previous + desired,
		}
	}

	pub fn push_with_read(&mut self, data: Expression, read: HashSet<ReadType>) {
		self.var_list.push(Slot { read, data });
	}

	pub fn push(&mut self, data: Expression) {
		self.push_with_read(data, HashSet::new());
	}

	pub fn push_with_single(&mut self, data: Expression) {
		let mut read = HashSet::new();
		let elem = match data {
			Expression::GetLocal(Local { var }) => ReadType::Local(var),
			Expression::GetGlobal(GetGlobal { var }) => ReadType::Global(var),
			Expression::LoadAt(LoadAt { memory, .. }) => ReadType::Memory(memory),
			_ => unreachable!(),
		};

		read.insert(elem);
		self.var_list.push(Slot { read, data });
	}

	pub fn pop_with_read(&mut self) -> (Expression, HashSet<ReadType>) {
		let var = self.var_list.pop().unwrap();

		(var.data, var.read)
	}

	pub fn pop(&mut self) -> Expression {
		self.pop_with_read().0
	}

	pub fn pop_len(&'_ mut self, len: usize) -> impl Iterator<Item = Expression> + '_ {
		let desired = self.len() - len;

		self.var_list.drain(desired..).map(|v| v.data)
	}

	pub fn push_temporaries(&mut self, num: usize) -> ResultList {
		let start = self.previous + self.len();
		let range = start..start + num;

		self.capacity = self.capacity.max(range.end);

		for var in range.clone() {
			let data = Expression::GetTemporary(Temporary { var });

			self.push(data);
		}

		ResultList::new(range.start, range.end)
	}

	pub fn push_temporary(&mut self) -> Temporary {
		self.push_temporaries(1).iter().next().unwrap()
	}

	// Return the alignment necessary for this block to branch out to a
	// another given stack frame
	pub fn get_br_alignment(&self, par_start: usize, par_result: usize) -> Align {
		let start = self.previous + self.len() - par_result;

		Align {
			new: par_start,
			old: start,
			length: par_result,
		}
	}

	// Try to leak a slot's value to a `SetTemporary` instruction,
	// adjusting the capacity and old index accordingly
	pub fn leak_into<P>(&mut self, code: &mut Vec<Statement>, predicate: P)
	where
		P: Fn(&Slot) -> bool,
	{
		for (i, old) in self.var_list.iter_mut().enumerate() {
			let var = self.previous + i;

			if old.is_temporary(var) || !predicate(old) {
				continue;
			}

			old.read.clear();

			let get = Expression::GetTemporary(Temporary { var });
			let set = Statement::SetTemporary(SetTemporary {
				var: Temporary { var },
				value: std::mem::replace(&mut old.data, get).into(),
			});

			self.capacity = self.capacity.max(var + 1);

			code.push(set);
		}
	}
}
