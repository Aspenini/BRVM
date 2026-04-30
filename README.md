# BRVM

BRVM is the Rust implementation of Brainrot Lang. It currently provides a source-to-bytecode compiler and a stack-based bytecode interpreter for `.brainrot` programs.

The current backend is BRBC bytecode. There is no native or AOT backend in the tree today; the LLVM AOT path is documented as a planned backend rather than a partial implementation.

## Quick Start

```bash
cargo build --release
```

Compile a source file to BRBC bytecode:

```bash
brvm compile examples/v1.brainrot -o examples/v1.brbc
```

Run bytecode:

```bash
brvm exec examples/v1.brbc
```

During development, the same commands can be run through Cargo:

```bash
cargo run -- compile examples/v1.brainrot -o /tmp/v1.brbc
cargo run -- exec /tmp/v1.brbc
```

## Brainrot Lang

A minimal program starts with `LOCK IN` and ends with `ITS OVER`.

```brainrot
LOCK IN
FANUMTAX aura FR "wsg"
FANUMTAX goon FR "WORLD!"
SAY aura 💀 " " 💀 goon
ITS OVER
```

Core features:

- Seven global braincells: `aura`, `peak`, `goon`, `mog`, `npc`, `sigma`, `gyatt`
- Numbers, strings, string concatenation, string repeat, arithmetic, and truthiness
- `FANUMTAX`, `DIDDLE`, `SAY`, `TOUCHY`, `ONGOD`, `NO CAP`, `SKIBIDI`, and `RETREAT`
- User functions with `TRALALERO ... TRALALA`
- Built-ins: `TOUCHY`, `TRANSFORM`, and `RIZZED`

See [Brainrot-Lang.md](Brainrot-Lang.md) for the language reference.

## BRVM Architecture

The runtime pipeline is:

```text
.brainrot source
  -> lexer
  -> parser AST
  -> BRBC bytecode compiler
  -> BRVM stack interpreter
```

Important implementation files:

- [src/lexer.rs](src/lexer.rs): tokenizes source, including emoji operators and comments
- [src/parser.rs](src/parser.rs): builds the AST for programs, statements, expressions, and functions
- [src/compiler.rs](src/compiler.rs): emits BRBC v4 bytecode
- [src/vm.rs](src/vm.rs): validates and executes bytecode
- [src/value.rs](src/value.rs): runtime value operations

See [docs/brvm.md](docs/brvm.md) for the bytecode/interpreter design and [docs/roadmap.md](docs/roadmap.md) for planned interpreter, LLVM AOT, and JIT work.

## Development

Run the test suite:

```bash
cargo test
```

Format code:

```bash
cargo fmt
```

The integration tests compile the bundled examples and exercise VM behavior such as prompt input, function calls, recursion, and string repeat.

## License

Licensed under the MIT license.
