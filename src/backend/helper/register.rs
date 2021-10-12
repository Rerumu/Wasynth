pub struct Register {
	pub last: u32,
	pub inner: u32,
	saved: Vec<u32>,
}

impl Register {
	pub fn new() -> Self {
		Self {
			last: 0,
			inner: 0,
			saved: vec![0],
		}
	}

	fn extend(&mut self) {
		self.last = self.last.max(self.inner);
	}

	pub fn save(&mut self) {
		self.saved.push(self.inner);
	}

	pub fn load(&mut self) {
		self.inner = self.saved.pop().unwrap();
	}

	pub fn push(&mut self, n: u32) -> u32 {
		let prev = self.inner;

		self.inner = self.inner.checked_add(n).unwrap();
		self.extend();

		prev
	}

	pub fn pop(&mut self, n: u32) -> u32 {
		self.inner = self.inner.checked_sub(n).unwrap();
		self.inner
	}
}
