# BRVM Architecture

BRVM is the reference runtime for Brainrot Lang. It is currently a bytecode compiler plus a stack-based bytecode interpreter.

## Current Pipeline

```text
source text
  -> lexer tokens
  -> parser AST
  -> BRBC bytecode
  -> stack VM execution
```

There is no native, LLVM, C, or WASM backend in the current implementation. The BRBC bytecode backend is the only executable backend and should remain the compatibility baseline while native compilation is added later.

## Source Frontend

The frontend has two stages:

- `src/lexer.rs`: converts source into tokens, including Brainrot keywords, emoji operators, comments, identifiers, numbers, strings, and braincells.
- `src/parser.rs`: converts tokens into an AST with expressions, statements, function definitions, and the main program block.

The parser currently reports simplified source locations. A future diagnostic pass should carry token spans through the AST so compile errors can point at the actual line and column.

## Bytecode Compiler

`src/compiler.rs` lowers the AST to BRBC v4 bytecode.

The compiler owns:

- Constant interning for numbers and strings
- Function symbol declaration before body compilation, which enables forward calls and recursion
- Global braincell loads/stores for main
- Local loads/stores for functions
- Jump backpatching for `ONGOD` and `SKIBIDI`
- Function jump relocation when function bodies are appended after main code

The compiler is the current backend. Future native backends should not replace the frontend directly; they should lower from a shared intermediate representation once that IR exists.

## BRBC Bytecode

BRBC files use this high-level layout:

```text
magic:          "BRBC"
version:        u16
flags:          u16
constant pool:  numbers and strings
function table: name constant, arity, local count, code offset
code section:   VM opcodes and operands
```

BRBC v4 stores all executable code in one code section. Main code starts at offset `0`; function entries point to absolute offsets inside the same section.

Opcode groups:

- Loading and storage: constants, globals, locals
- Arithmetic and string operations
- I/O: print, input, prompted input
- Control flow: absolute jump and jump-if-false
- Calls: built-ins and user functions
- Return and halt

## Interpreter

`src/vm.rs` loads bytecode, validates section boundaries, then runs opcodes with:

- A value stack
- Seven optional global braincells
- A call stack of return addresses and local slots
- A constant pool
- A function metadata table
- Injectable input/output streams for tests and embedders

The interpreter now treats malformed bytecode reads as runtime errors instead of silently decoding missing operands as zero. This matters for reliability now and for future compiled backends, because the bytecode format can be verified before native lowering.

## Runtime Values

`src/value.rs` contains dynamic value operations for:

- Numeric arithmetic
- String concatenation through `💀`
- String repeat through `😏`
- Print formatting

Values are intentionally small today: `Number(f64)` and `String(Rc<String>)`. A future LLVM backend will need a stable runtime ABI for this dynamic value representation before it can emit object files.

## Improvement Backlog

High-value interpreter and compiler improvements:

- Add token spans and source ranges to lexer/parser errors
- Add a BRBC disassembler for debugging generated code
- Add a bytecode verifier before execution
- Replace raw opcode numbers with a typed opcode definition used by compiler, VM, verifier, and disassembler
- Add benchmark programs and track interpreter throughput
- Pre-size stacks and local vectors from bytecode metadata where possible
- Split parsing, IR lowering, bytecode emission, and native backend lowering into explicit stages
- Add a shared mid-level IR before starting LLVM work
