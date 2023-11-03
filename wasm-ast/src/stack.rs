use crate::{
	node::{
		Align, Expression, GetGlobal, LoadAt, Local, ResultList, SetTemporary, Statement, Temporary,
	},
	visit::{Driver, Visitor},
};

pub struct ReadGet<A, B, C> {
	has_local: A,
	has_global: B,
	has_memory: C,
	result: bool,
}

impl<A, B, C> ReadGet<A, B, C>
where
	A: Fn(Local) -> bool,
	B: Fn(GetGlobal) -> bool,
	C: Fn(&LoadAt) -> bool,
{
	pub fn run<D: Driver<Self>>(node: &D, has_local: A, has_global: B, has_memory: C) -> bool {
		let mut visitor = Self {
			has_local,
			has_global,
			has_memory,
			result: false,
		};

		node.accept(&mut visitor);

		visitor.result
	}
}

impl<A, B, C> Visitor for ReadGet<A, B, C>
where
	A: Fn(Local) -> bool,
	B: Fn(GetGlobal) -> bool,
	C: Fn(&LoadAt) -> bool,
{
	fn visit_get_global(&mut self, get_global: GetGlobal) {
		self.result |= (self.has_global)(get_global);
	}

	fn visit_load_at(&mut self, load_at: &LoadAt) {
		self.result |= (self.has_memory)(load_at);
	}

	fn visit_get_local(&mut self, local: Local) {
		self.result |= (self.has_local)(local);
	}
}

#[derive(Default)]
pub struct Stack {
	var_list: Vec<Expression>,
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

	pub fn push(&mut self, data: Expression) {
		self.var_list.push(data);
	}

	pub fn pop(&mut self) -> Expression {
		self.var_list.pop().unwrap()
	}

	pub fn pop_len(&'_ mut self, len: usize) -> impl Iterator<Item = Expression> + '_ {
		let desired = self.len() - len;

		self.var_list.drain(desired..)
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
		P: Fn(&Expression) -> bool,
	{
		for (i, old) in self.var_list.iter_mut().enumerate() {
			let var = self.previous + i;
			let is_temporary =
				matches!(old, Expression::GetTemporary(temporary) if temporary.var() == var);

			if is_temporary || !predicate(old) {
				continue;
			}

			let get = Expression::GetTemporary(Temporary { var });
			let set = Statement::SetTemporary(SetTemporary {
				var: Temporary { var },
				value: std::mem::replace(old, get).into(),
			});

			self.capacity = self.capacity.max(var + 1);

			code.push(set);
		}
	}
}
