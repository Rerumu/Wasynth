pub static RUNTIME: &str = concat!(
	"local I64 = (function() ",
	include_str!("../runtime/numeric.lua"),
	" end)()\n",
	include_str!("../runtime/runtime.lua")
);

pub use translator::{from_inst_list, from_module};

mod analyzer;
mod backend;
mod translator;
