pub static RUNTIME: &str = include_str!("../runtime/runtime.lua");

pub use translator::translate;

mod analyzer;
mod backend;
mod translator;
