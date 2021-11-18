// Translation is done in levels.
// Level 1 handles user logic and WASM instructions.
// Level 2 handles setup for functions.
// Level 3 handles initialization of the module.

mod level_1;
mod level_2;
pub mod level_3;
