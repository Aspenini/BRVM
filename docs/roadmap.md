# Backend And Performance Roadmap

This roadmap keeps BRBC bytecode as the stable runtime contract while making room for an eventual LLVM AOT backend and a later JIT.

## Current State

BRVM currently has one real backend:

- BRBC bytecode emission in `src/compiler.rs`
- BRBC interpretation in `src/vm.rs`

No native backend exists in the tree. Unsupported native/AOT claims should stay out of user docs until an LLVM backend can compile and link real programs.

## Phase 1: Harden The Interpreter

Goal: make the current VM reliable enough to be the reference behavior for every later backend.

Work items:

- Keep expanding integration tests around examples, function calls, recursion, strings, input, and malformed bytecode
- Add bytecode verification before execution
- Add a human-readable disassembler
- Replace raw opcode literals with a single typed opcode table
- Add source spans to diagnostics
- Add benchmark fixtures for loops, calls, string operations, input-free numeric code, and recursion

Performance items:

- Avoid unnecessary value clones in hot opcodes
- Pre-size the value stack and call stack when metadata allows it
- Consider specialized numeric opcodes after type profiling exists
- Keep I/O outside hot paths so benchmarks measure VM work

## Phase 2: Introduce A Shared IR

Goal: stop treating bytecode as the only compiler target.

Add a mid-level IR between the parser AST and backend emitters:

```text
AST -> semantic analysis -> BRIR -> backend
```

BRIR should represent:

- Function symbols, arity, and locals
- Basic blocks and explicit branches
- Dynamic value operations
- Built-in calls
- Global braincell access
- Return and halt

This IR should be easier to verify than AST and easier to lower than stack bytecode. Bytecode can then become one backend that lowers from BRIR.

## Phase 3: LLVM AOT Backend Setup

Goal: add native object/executable generation without compromising the interpreter.

Recommended shape:

```text
AST -> semantic analysis -> BRIR
                          -> BRBC bytecode backend
                          -> LLVM AOT backend
```

Implementation plan:

- Gate the backend behind a Cargo feature such as `llvm-backend`
- Use a Rust LLVM binding such as `inkwell`, pinned to a documented LLVM version
- Define a small C-compatible runtime ABI for dynamic `Value` operations
- Lower BRIR functions to LLVM functions
- Call runtime helpers for dynamic add/sub/mul/div, truthiness, print, input, and string allocation
- Emit object files first, then add executable linking
- Add a CLI command only after object emission works, for example `brvm build input.brainrot -o program`

Native backend non-goals at this stage:

- No ad hoc C transpiler
- No separate semantics from the interpreter
- No LLVM dependency in the default build
- No native backend documentation that implies production readiness before examples compile and run

## Phase 4: JIT Jump

Goal: upgrade the interpreter into an adaptive JIT path for hot code while keeping interpreter fallback.

The JIT should reuse the LLVM lowering from the AOT backend through LLVM ORC JIT, or another backend only if LLVM proves too heavy for interactive compilation.

Plan:

- Add counters for function entries and backward loop edges in the interpreter
- Keep running cold code in the interpreter
- When a function or loop crosses a threshold, lower its BRIR to native code
- Guard dynamic assumptions, such as numeric-only hot paths
- Deopt or fall back to the interpreter when assumptions fail
- Keep built-ins and heap operations as calls into the same runtime ABI used by AOT

The first JIT should be a baseline JIT, not an optimizing compiler. The priority is correctness and low integration risk. Once baseline JIT works, add type-specialized traces for numeric loops and string-heavy paths.

## Phase 5: Optimization

After bytecode, AOT, and JIT share one IR, add optimizations in front of both native backends:

- Constant folding
- Dead code elimination after unconditional `RETREAT` or halt
- Local slot reuse
- Basic block simplification
- Numeric fast paths with runtime guards
- Tail-call handling for simple recursion

## Definition Of Done For LLVM AOT

LLVM AOT should not be considered present until all are true:

- It is behind an explicit Cargo feature
- It compiles the bundled examples that do not require interactive input
- It has tests comparing interpreter output and native output
- It documents required LLVM version and install steps
- It leaves bytecode compile/exec behavior unchanged

## Definition Of Done For JIT

The JIT should not replace the interpreter until all are true:

- It can be disabled with a CLI flag or environment variable
- It falls back to the interpreter on unsupported code
- It passes the same behavioral tests as bytecode execution
- It includes benchmarks showing real improvement on hot loops or hot functions
- It exposes enough tracing/logging to debug compiled regions
