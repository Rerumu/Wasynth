## Note of Discontinuation

This project served well as a proof of concept and initial testing ground for what I wanted to work on. It will not receive any further updates due to accumulating technical debt and the rigid design that makes work on it difficult. [Spider](https://github.com/SovereignSatellite/Spider) will be its successor project, where development will continue.

## Wasynth

This is a WebAssembly translation tool and library for arbitrary languages. It contains several modules for different purposes as outlined below.

* `wasm-ast` handles creating abstract syntax trees which can be used to inspect and act on WebAssembly code.
* `codegen/*` handles individual code generation libraries that consume the syntax trees.
* `dev-test/tests/*` handles testing the code generation against the standard test suite.
* `dev-test/fuzz_targets/*` handles testing syntax tree building through fuzzing of pseudo-random data.

## Code Generation

The code generation libraries also offer a simple binary utility for translating to source. These can be built or installed by using the `--path codegen/language --bin wasm2language` Cargo flags.

|          |                |                       |
|----------|----------------|-----------------------|
| LuaJIT   | :green_circle: | Minimum version 2.1.0 |
| Luau     | :green_circle: |                       |
