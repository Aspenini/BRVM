# BRVM - Brainrot Virtual Machine

A compiler and virtual machine for the Brainrot programming language, written in Rust.

## Installation

```bash
cargo install brvm
```

Or install from source:

```bash
git clone https://github.com/yourusername/brvm.git
cd brvm
cargo build --release
```

## Usage

### Compiling Brainrot code

```bash
brvm compile input.brainrot -o output.brbc
```

If no output is specified, it defaults to `input.brbc` in the same directory.

### Running bytecode

```bash
brvm exec output.brbc
```

## Language Features (v3)

### Braincells

Seven global variables: `aura`, `peak`, `goon`, `mog`, `npc`, `sigma`, `gyatt`

### Statements

- `FANUMTAX <cell> FR <expr>` - Assign a value to a braincell
- `SAY <expr>` - Print a value with newline
- `ONGOD <expr> ... DEADASS` - If statement
- `ONGOD <expr> ... NO CAP ... DEADASS` - If/else statement
- `SKIBIDI <expr> ... RIZZUP` - While loop

### Expressions

- Numbers (f64): `42`, `3.14`
- Strings: `"hello"`, `"wsg WORLD!"`
- Variables: braincell names
- Binary operators:
  - `💀` - Addition / String concatenation
  - `😭` - Subtraction
  - `😏` - Multiplication
  - `🚡` - Division
- Function calls:
  - `TOUCHY()` - Read input from stdin
  - `TOUCHY("prompt: ")` - Print prompt and read input

### Examples

#### Hello World

```brainrot
LOCK IN
SAY "wsg WORLD!"
ITS OVER
```

#### If/Else

```brainrot
LOCK IN
FANUMTAX sigma FR 0
ONGOD sigma
  SAY "nonzero"
NO CAP
  SAY "zero"
DEADASS
ITS OVER
```

#### While Loop

```brainrot
LOCK IN
FANUMTAX gyatt FR 3
SKIBIDI gyatt
  SAY gyatt
  FANUMTAX gyatt FR gyatt 😭 1
RIZZUP
ITS OVER
```

Output: `3\n2\n1`

#### Interactive Input

```brainrot
LOCK IN
FANUMTAX aura FR TOUCHY("name: ")
ONGOD aura
  SAY "hi " 💀 aura
NO CAP
  SAY "no name provided"
DEADASS
ITS OVER
```

## Truthiness

- `Number(0.0)` → false
- `String("")` → false
- Everything else → true

## License

Licensed under MIT license

at your option.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

