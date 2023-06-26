use std::collections::HashMap;

use wasmparser::{
	BlockType, Data, Element, Export, ExternalKind, FunctionBody, Global, Import, LocalsReader,
	MemoryType, Name, NameSectionReader, Parser, Payload, Result, TableType, Type, TypeRef,
	ValType,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum External {
	Func,
	Table,
	Memory,
	Global,
	Tag,
}

impl From<TypeRef> for External {
	fn from(value: TypeRef) -> Self {
		match value {
			TypeRef::Func(_) => Self::Func,
			TypeRef::Table(_) => Self::Table,
			TypeRef::Memory(_) => Self::Memory,
			TypeRef::Global(_) => Self::Global,
			TypeRef::Tag(_) => Self::Tag,
		}
	}
}

impl From<ExternalKind> for External {
	fn from(value: ExternalKind) -> Self {
		match value {
			ExternalKind::Func => Self::Func,
			ExternalKind::Table => Self::Table,
			ExternalKind::Memory => Self::Memory,
			ExternalKind::Global => Self::Global,
			ExternalKind::Tag => Self::Tag,
		}
	}
}

pub(crate) fn read_checked<T, I>(reader: I) -> Result<Vec<T>>
where
	I: IntoIterator<Item = Result<T>>,
{
	reader.into_iter().collect()
}

pub(crate) fn read_checked_locals(reader: LocalsReader) -> Result<Vec<ValType>> {
	read_checked(reader).map(|locals| {
		let convert = |(a, b)| std::iter::repeat(b).take(usize::try_from(a).unwrap());

		locals.into_iter().flat_map(convert).collect()
	})
}

pub struct Module<'a> {
	type_section: Vec<Type>,
	import_section: Vec<Import<'a>>,
	func_section: Vec<u32>,
	table_section: Vec<TableType>,
	memory_section: Vec<MemoryType>,
	global_section: Vec<Global<'a>>,
	export_section: Vec<Export<'a>>,
	element_section: Vec<Element<'a>>,
	data_section: Vec<Data<'a>>,
	code_section: Vec<FunctionBody<'a>>,

	name_section: HashMap<u32, &'a str>,

	start_section: Option<u32>,
}

impl<'a> Module<'a> {
	/// # Errors
	///
	/// Returns a `BinaryReaderError` if any module section is malformed.
	pub fn try_from_data(data: &'a [u8]) -> Result<Self> {
		let mut temp = Module {
			type_section: Vec::new(),
			import_section: Vec::new(),
			func_section: Vec::new(),
			table_section: Vec::new(),
			memory_section: Vec::new(),
			global_section: Vec::new(),
			export_section: Vec::new(),
			element_section: Vec::new(),
			data_section: Vec::new(),
			code_section: Vec::new(),
			name_section: HashMap::new(),
			start_section: None,
		};

		temp.load_data(data)?;
		Ok(temp)
	}

	fn load_data(&mut self, data: &'a [u8]) -> Result<()> {
		for payload in Parser::new(0).parse_all(data) {
			match payload? {
				Payload::TypeSection(v) => self.type_section = read_checked(v)?,
				Payload::ImportSection(v) => self.import_section = read_checked(v)?,
				Payload::FunctionSection(v) => self.func_section = read_checked(v)?,
				Payload::TableSection(v) => self.table_section = read_checked(v)?,
				Payload::MemorySection(v) => self.memory_section = read_checked(v)?,
				Payload::GlobalSection(v) => self.global_section = read_checked(v)?,
				Payload::ExportSection(v) => self.export_section = read_checked(v)?,
				Payload::ElementSection(v) => self.element_section = read_checked(v)?,
				Payload::DataSection(v) => self.data_section = read_checked(v)?,
				Payload::CodeSectionEntry(v) => {
					self.code_section.push(v);
				}
				Payload::StartSection { func, .. } => {
					self.start_section = Some(func);
				}
				Payload::CustomSection(v) if v.name() == "name" => {
					for name in NameSectionReader::new(v.data(), v.data_offset()) {
						if let Name::Function(map) = name? {
							let mut iter = map.into_iter();
							while let Some(Ok(elem)) = iter.next() {
								self.name_section.insert(elem.index, elem.name);
							}
						}
					}
				}
				_ => {}
			}
		}

		Ok(())
	}

	#[must_use]
	pub fn import_count(&self, ext: External) -> usize {
		let predicate = |v: &&Import| External::from(v.ty) == ext;

		self.import_section.iter().filter(predicate).count()
	}

	#[must_use]
	pub fn function_space(&self) -> usize {
		self.import_count(External::Func) + self.func_section.len()
	}

	#[must_use]
	pub fn table_space(&self) -> usize {
		self.import_count(External::Table) + self.table_section.len()
	}

	#[must_use]
	pub fn memory_space(&self) -> usize {
		self.import_count(External::Memory) + self.memory_section.len()
	}

	#[must_use]
	pub fn global_space(&self) -> usize {
		self.import_count(External::Global) + self.global_section.len()
	}

	#[must_use]
	pub fn type_section(&self) -> &[Type] {
		&self.type_section
	}

	#[must_use]
	pub fn import_section(&self) -> &[Import] {
		&self.import_section
	}

	#[must_use]
	pub fn func_section(&self) -> &[u32] {
		&self.func_section
	}

	#[must_use]
	pub fn table_section(&self) -> &[TableType] {
		&self.table_section
	}

	#[must_use]
	pub fn memory_section(&self) -> &[MemoryType] {
		&self.memory_section
	}

	#[must_use]
	pub fn global_section(&self) -> &[Global] {
		&self.global_section
	}

	#[must_use]
	pub fn export_section(&self) -> &[Export] {
		&self.export_section
	}

	#[must_use]
	pub fn element_section(&self) -> &[Element] {
		&self.element_section
	}

	#[must_use]
	pub fn data_section(&self) -> &[Data] {
		&self.data_section
	}

	#[must_use]
	pub fn code_section(&self) -> &[FunctionBody] {
		&self.code_section
	}

	#[must_use]
	pub const fn name_section(&self) -> &HashMap<u32, &'a str> {
		&self.name_section
	}

	#[must_use]
	pub const fn start_section(&self) -> Option<u32> {
		self.start_section
	}
}

pub struct TypeInfo<'a> {
	type_list: &'a [Type],
	func_list: Vec<usize>,
}

impl<'a> TypeInfo<'a> {
	#[must_use]
	pub fn from_module(wasm: &'a Module) -> Self {
		let mut temp = Self {
			type_list: &wasm.type_section,
			func_list: Vec::new(),
		};

		temp.load_import_list(&wasm.import_section);
		temp.load_func_list(&wasm.func_section);
		temp
	}

	fn load_import_list(&mut self, list: &[Import]) {
		let iter = list
			.iter()
			.copied()
			.filter_map(|v| match v.ty {
				TypeRef::Func(v) => Some(v),
				_ => None,
			})
			.map(|v| usize::try_from(v).unwrap());

		self.func_list.extend(iter);
	}

	fn load_func_list(&mut self, list: &[u32]) {
		let iter = list.iter().copied().map(|v| usize::try_from(v).unwrap());

		self.func_list.extend(iter);
	}

	pub(crate) fn by_type_index(&self, index: usize) -> (usize, usize) {
		let Type::Func(ty) = &self.type_list[index];

		(ty.params().len(), ty.results().len())
	}

	pub(crate) fn by_func_index(&self, index: usize) -> (usize, usize) {
		let adjusted = self.func_list[index];

		self.by_type_index(adjusted)
	}

	pub(crate) fn by_block_type(&self, ty: BlockType) -> (usize, usize) {
		match ty {
			BlockType::Empty => (0, 0),
			BlockType::Type(_) => (0, 1),
			BlockType::FuncType(i) => {
				let id = i.try_into().unwrap();

				self.by_type_index(id)
			}
		}
	}
}
