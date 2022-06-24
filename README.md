# Wasynth

This is a WebAssembly translation tool and library for arbitrary languages. It contains several modules for different purposes as outlined below.

* `wasm-ast` handles creating abstract syntax trees which can be used to inspect and act on WebAssembly code.
* `codegen/*` handles individual code generation libraries that consume the syntax trees.
* `dev-test/tests/*` handles testing the code generation against the standard test suite.
* `dev-test/fuzz_targets/*` handles testing syntax tree building through fuzzing of pseudo-random data.

## Code Generation

The code generation libraries also offer a simple binary utility for translating to source. These can be built or installed by using the `--path codegen/language --bin wasm2language` Cargo flags.

|          |                 |                       |
|----------|-----------------|-----------------------|
| LuaJIT   | :green_circle:  | Minimum version 2.1.0 |
| Luau     | :yellow_circle: |                       |
