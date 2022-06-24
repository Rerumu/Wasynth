pub static RUNTIME: &str = include_str!("../runtime/runtime.lua");
pub static NUMERIC: &str = include_str!("../runtime/numeric.lua");

pub use translator::{from_inst_list, from_module_typed, from_module_untyped};

mod analyzer;
mod backend;
mod translator;
