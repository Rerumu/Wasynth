use std::{
	collections::HashSet,
	io::{Result, Write},
};

use generational_arena::Index;

use crate::graph::node::{Edge, Gamma, Lambda, Node, Simple, Theta};

use super::label::label_of_simple;

pub struct Writer<'s, 'a> {
	name: &'s str,
	lambda: &'a Lambda,
	added: HashSet<usize>,
}

impl<'s, 'a> Writer<'s, 'a> {
	pub fn new(name: &'s str, lambda: &'a Lambda) -> Self {
		Self {
			lambda,
			name,
			added: HashSet::new(),
		}
	}

	pub fn write(&mut self, w: &mut dyn Write) -> Result<()> {
		self.added.clear();

		writeln!(w, "digraph {} {{", self.name)?;
		writeln!(w, "node [style = filled, shape = box];")?;
		writeln!(w, "style = filled;")?;

		self.write_all(w)?;
		self.write_unreachable(w)?;

		writeln!(w, "}}")
	}

	fn write_all(&mut self, w: &mut dyn Write) -> Result<()> {
		writeln!(w, "subgraph cluster_reachable {{")?;
		writeln!(w, "node [fillcolor = aliceblue];")?;
		writeln!(w, "fillcolor = snow;")?;

		for (i, &v) in self.lambda.list_out.data.iter().enumerate() {
			self.maybe_add_node(v.node(), w)?;

			let id = v.node().into_raw_parts().0;

			writeln!(w, "Node_{id} -> End [taillabel = {i}];")?;
		}

		writeln!(w, "}}")
	}

	fn write_unreachable(&mut self, w: &mut dyn Write) -> Result<()> {
		writeln!(w, "subgraph cluster_unreachable {{")?;
		writeln!(w, "node [fillcolor = lightpink];")?;
		writeln!(w, "fillcolor = grey;")?;

		for (i, _) in &self.lambda.arena {
			self.maybe_add_node(i, w)?;
		}

		writeln!(w, "}}")
	}

	fn add_edge(&mut self, to: Index, from: Edge, w: &mut dyn Write) -> Result<()> {
		let port = from.port();
		let to_id = to.into_raw_parts().0;
		let from_id = from.node().into_raw_parts().0;

		writeln!(w, "Node_{from_id} -> Node_{to_id} [taillabel = {port}];")?;

		self.maybe_add_node(from.node(), w)
	}

	fn add_list(&mut self, start: Index, end: &[Edge], w: &mut dyn Write) -> Result<()> {
		end.iter().try_for_each(|&e| self.add_edge(start, e, w))
	}

	fn add_simple(&mut self, start: Index, node: &Simple, w: &mut dyn Write) -> Result<()> {
		match node {
			Simple::Unreachable(v) => {
				self.add_edge(start, v.memory_order, w)?;
				self.add_edge(start, v.global_order, w)?;
			}
			Simple::GetTableElement(v) => {
				self.add_edge(start, v.index, w)?;
			}
			Simple::Call(v) => {
				self.add_edge(start, v.function, w)?;
				self.add_list(start, &v.argument_list, w)?;
				self.add_edge(start, v.memory_order, w)?;
				self.add_edge(start, v.global_order, w)?;
			}
			Simple::GlobalGet(v) => {
				self.add_edge(start, v.order, w)?;
			}
			Simple::GlobalSet(v) => {
				self.add_edge(start, v.value, w)?;
				self.add_edge(start, v.order, w)?;
			}
			Simple::Load(v) => {
				self.add_edge(start, v.pointer, w)?;
				self.add_edge(start, v.order, w)?;
			}
			Simple::Store(v) => {
				self.add_edge(start, v.pointer, w)?;
				self.add_edge(start, v.value, w)?;
				self.add_edge(start, v.order, w)?;
			}
			Simple::MemorySize(v) => {
				self.add_edge(start, v.order, w)?;
			}
			Simple::MemoryGrow(v) => {
				self.add_edge(start, v.delta, w)?;
				self.add_edge(start, v.order, w)?;
			}
			Simple::UnOp(v) => {
				self.add_edge(start, v.rhs, w)?;
			}
			Simple::BinOp(v) => {
				self.add_edge(start, v.lhs, w)?;
				self.add_edge(start, v.rhs, w)?;
			}
			Simple::CmpOp(v) => {
				self.add_edge(start, v.lhs, w)?;
				self.add_edge(start, v.rhs, w)?;
			}
			Simple::CastOp(v) => {
				self.add_edge(start, v.rhs, w)?;
			}
			_ => {}
		}

		Ok(())
	}

	fn add_gamma(&mut self, start: Index, node: &Gamma, w: &mut dyn Write) -> Result<()> {
		let id = start.into_raw_parts().0;

		for e in &node.list_out {
			self.add_list(e.0.node(), &node.list_in.data, w)?;
		}

		writeln!(w, "subgraph cluster_{id} {{")?;
		writeln!(w, "fillcolor = dodgerblue;")?;
		writeln!(
			w,
			"Node_{id} [shape = house, group = Gamma, label = Gamma];"
		)?;

		self.add_edge(start, node.condition, w)?;

		for (i, e) in node.list_out.iter().enumerate() {
			writeln!(w, "subgraph cluster_{id}_{i} {{")?;
			writeln!(w, "fillcolor = deepskyblue;")?;
			writeln!(w, "label = {i};")?;

			self.add_list(start, &e.1.data, w)?;

			writeln!(w, "}}")?;
		}

		writeln!(w, "}}")
	}

	fn add_theta(&mut self, start: Index, node: &Theta, w: &mut dyn Write) -> Result<()> {
		let id = start.into_raw_parts().0;

		self.add_list(node.param_edge.node(), &node.list_in.data, w)?;

		writeln!(w, "subgraph cluster_{id} {{")?;
		writeln!(w, "fillcolor = aquamarine;")?;
		writeln!(
			w,
			"Node_{id} [shape = parallelogram, group = Theta, label = Theta];"
		)?;

		self.add_list(start, &node.list_out.data, w)?;
		self.add_edge(start, node.condition, w)?;

		writeln!(w, "}}")
	}

	fn add_node(&mut self, start: Index, w: &mut dyn Write) -> Result<()> {
		let id = start.into_raw_parts().0;
		let node = match self.lambda.arena.get(start) {
			Some(node) => node,
			None => return writeln!(w, "Node_{id} [fillcolor = lightcoral, shape = square];"),
		};

		match node {
			Node::Simple(simple) => {
				let label = label_of_simple(simple);

				writeln!(w, "Node_{id} [group = Simple, label = \"{label}\"];")?;
				self.add_simple(start, simple, w)
			}
			Node::Gamma(gamma) => self.add_gamma(start, gamma, w),
			Node::Theta(theta) => self.add_theta(start, theta, w),
		}
	}

	fn maybe_add_node(&mut self, start: Index, w: &mut dyn Write) -> Result<()> {
		let id = start.into_raw_parts().0;

		if self.added.insert(id) {
			self.add_node(start, w)
		} else {
			Ok(())
		}
	}
}
