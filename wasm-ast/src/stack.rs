use std::collections::HashSet;

use crate::node::{
	Align, Expression, GetGlobal, GetLocal, GetTemporary, LoadAt, SetTemporary, Statement,
};

#[derive(PartialEq, Eq, Hash)]
pub enum ReadType {
	Local(usize),
	Global(usize),
	Memory(usize),
}

pub struct Slot {
	pub read: HashSet<ReadType>,
	pub data: Expression,
}

#[derive(Default)]
pub struct Stack {
	pub var_list: Vec<Slot>,
	pub capacity: usize,
	pub previous: usize,
}

impl Stack {
	pub fn split_last(&mut self, len: usize) -> Self {
		let desired = self.var_list.len() - len;
		let content = self.var_list.split_off(desired);

		Self {
			var_list: content,
			capacity: self.capacity,
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
			Expression::GetLocal(GetLocal { var }) => ReadType::Local(var),
			Expression::GetGlobal(GetGlobal { var }) => ReadType::Global(var),
			Expression::LoadAt(LoadAt { .. }) => ReadType::Memory(0),
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
		let desired = self.var_list.len() - len;

		self.var_list.drain(desired..).map(|v| v.data)
	}

	pub fn push_temporary(&mut self, num: usize) {
		let len = self.var_list.len() + self.previous;

		for var in len..len + num {
			let data = Expression::GetTemporary(GetTemporary { var });

			self.push(data);
		}

		self.capacity = self.capacity.max(len + num);
	}

	// Try to leak a slot's value to a `SetTemporary` instruction,
	// adjusting the capacity and old index accordingly
	pub fn leak_at(&mut self, index: usize) -> Option<Statement> {
		let old = &mut self.var_list[index];
		let var = self.previous + index;

		if old.data.is_temporary(var) {
			return None;
		}

		old.read.clear();

		let get = Expression::GetTemporary(GetTemporary { var });
		let set = Statement::SetTemporary(SetTemporary {
			var,
			value: std::mem::replace(&mut old.data, get),
		});

		self.capacity = self.capacity.max(var + 1);

		Some(set)
	}

	// Return the alignment necessary for this block to branch out to a
	// another given stack frame
	pub fn get_br_alignment(&self, par_start: usize, par_result: usize) -> Align {
		let start = self.var_list.len() + self.previous - par_result;

		Align {
			new: par_start,
			old: start,
			length: par_result,
		}
	}
}
