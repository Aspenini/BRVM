# Brainrot Lang

Brainrot Lang is a small dynamic language built around Brainrot-flavored keywords and emoji operators. BRVM compiles Brainrot source into BRBC bytecode and runs it on a stack VM.

## Program Shape

Every program has one main block. Function definitions, when present, must appear before `LOCK IN`.

```brainrot
LOCK IN
SAY "wsg WORLD!"
ITS OVER
```

Comments start with `🖕` and continue to the end of the line.

```brainrot
🖕 this line is ignored
```

## Values

Brainrot currently has two runtime value types:

- Number: stored as `f64`, for example `42` or `3.14`
- String: double-quoted UTF-8 text, with escapes such as `\"`, `\\`, `\n`, and `\t`

Truthiness:

- `0` is false
- `""` is false
- every other number or string is true

## Identifiers And Braincells

Identifiers can contain ASCII letters, digits, and underscores, but cannot start with a digit.

Main-program storage is limited to seven global braincells:

- `aura`
- `peak`
- `goon`
- `mog`
- `npc`
- `sigma`
- `gyatt`

Functions use local variables. Function assignments are local even when the local name matches a braincell. A function can still read a global braincell if that name has not been shadowed by a local.

## Statements

### Assign

`FANUMTAX <name> FR <expr>` assigns an expression to a braincell in main or a local in a function.

```brainrot
FANUMTAX sigma FR 10 😏 2
```

### Copy

`DIDDLE <name> FR <expr>` evaluates the expression and stores the result.

```brainrot
DIDDLE gyatt FR sigma
```

### Print

`SAY <expr>` prints a value followed by a newline.

```brainrot
SAY "sum: " 💀 sigma
```

### Halt

`YOUSHALLNOTPASS` stops execution.

```brainrot
YOUSHALLNOTPASS
```

## Input

`TOUCHY()` reads one line from stdin and returns it without the trailing newline.

`TOUCHY(<prompt>)` writes the prompt first, flushes output, then reads one line.

```brainrot
LOCK IN
FANUMTAX aura FR TOUCHY("name: ")
SAY "hi " 💀 aura
ITS OVER
```

## Operators

| Brainrot | Meaning |
| --- | --- |
| `💀` | number addition or string concatenation |
| `😭` | number subtraction |
| `😏` | number multiplication or string repeat |
| `🚡` | number division |

Precedence: `😏` and `🚡` bind before `💀` and `😭`.

```brainrot
FANUMTAX mog FR 10 😏 2 💀 5      🖕 25
FANUMTAX npc FR "hi" 💀 "!"       🖕 hi!
FANUMTAX aura FR "ha" 😏 3        🖕 hahaha
```

String repeat requires a non-negative whole-number repeat count.

Parentheses are supported for function calls but not for grouping arbitrary arithmetic expressions. Split complex expressions across assignments when needed.

## Control Flow

### If / Else

Use `ONGOD` for the condition, optional `NO CAP` for else, and `DEADASS` to close the block.

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

### While

Use `SKIBIDI` to start a loop and `RIZZUP` to close it.

```brainrot
LOCK IN
FANUMTAX gyatt FR 3
SKIBIDI gyatt
  SAY gyatt
  FANUMTAX gyatt FR gyatt 😭 1
RIZZUP
ITS OVER
```

Output:

```text
3
2
1
```

## Functions

Define functions with `TRALALERO` and close them with `TRALALA`.

Return with `RETREAT <expr>`. If a function reaches the end without `RETREAT`, it returns `""`.

```brainrot
TRALALERO greet(name)
  FANUMTAX message FR "wsg " 💀 name 💀 "!"
  RETREAT message
TRALALA

LOCK IN
SAY greet("sigma")
ITS OVER
```

Function calls can use the direct form:

```brainrot
SAY greet("sigma")
```

The older explicit call form is also accepted:

```brainrot
SAY ring yas greet("sigma")
```

Functions may call functions defined later in the file and may call themselves recursively.

```brainrot
TRALALERO fact(n)
  ONGOD n 😭 1
    RETREAT n 😏 fact(n 😭 1)
  NO CAP
    RETREAT 1
  DEADASS
TRALALA

LOCK IN
SAY fact(5)
ITS OVER
```

## Built-Ins

### TOUCHY

Reads a line of input.

```brainrot
FANUMTAX aura FR TOUCHY("name: ")
```

### TRANSFORM

Converts a string to a number.

```brainrot
FANUMTAX sigma FR TRANSFORM("42")
```

### RIZZED

Returns the character length of a string.

```brainrot
FANUMTAX sigma FR RIZZED("hello")
```

## Errors

Common compile-time errors:

- Missing `LOCK IN` or `ITS OVER`
- Unknown main-program braincell
- Malformed function parameter or argument lists
- Mismatched block terminators
- Undefined function calls

Common runtime errors:

- Reading an unset braincell or local
- Stack underflow from malformed bytecode
- Constant, local, function, or jump index out of bounds
- Invalid numeric conversion in `TRANSFORM`
- Division by zero
- Invalid string repeat count

## Cheatsheet

```text
LOCK IN ... ITS OVER                  main program
🖕 comment                            comment
FANUMTAX name FR expr                 assign
DIDDLE name FR expr                   copy value
SAY expr                              print
TOUCHY() / TOUCHY("prompt")           input
ONGOD expr ... NO CAP ... DEADASS     if / else
SKIBIDI expr ... RIZZUP               while
TRALALERO name(args) ... TRALALA      function
RETREAT expr                          return
ring yas name(args)                   explicit function call
```
