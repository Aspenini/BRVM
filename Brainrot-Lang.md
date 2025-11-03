# Brainrot Lang

**Brainrot Lang** is an interpreted programming language aimed at using only brainrot terms!

```brainrot
LOCK IN
FANUMTAX gyatt FR "wsg"
FANUMTAX goon FR "WORLD!"
FANUMTAX sigma FR gyatt 💀 goon
SAY sigma
ITS OVER
```

---

## Program Structure
Programs must start with `LOCK IN` and end with `ITS OVER`.

```brainrot
LOCK IN
🖕 ...code
ITS OVER
```

---

## Comments
Rather than the typical `#`, comments are written with `🖕`.

```brainrot
🖕 this is a comment! this code sucks
```

---

## Math
Mathematical symbols are replaced with emojis/terms:  

- `💀 = +`  
- `😭 = -`  
- `😏 = ×`  
- `🚡 = ÷`  
- `FR = =`  

Notes:
- `💀` also concatenates strings: `"hi" 💀 " there"` → `hi there`
- String repeat with `😏`: `"ha" 😏 3` → `hahaha`
- Division returns a number; avoid dividing by zero (runtime error)

---

## Braincells
Data can be stored in any of the **7 braincells**:

- `aura`  
- `peak`  
- `goon`  
- `mog`  
- `npc`  
- `sigma`  
- `gyatt`  

Example:  

```brainrot
FANUMTAX peak FR 2
```

---

## I/O
`SAY` will print values.
'TOUCHY()` will get input.

```brainrot
LOCK IN
FANUMTAX goon FR 69
SAY goon
ITS OVER
```
Output:
```
69
```

---

## Data and Copy
- `FANUMTAX` declares/assigns a value to a braincell.  
- `DIDDLE` copies the value from one braincell to another.

```brainrot
LOCK IN
FANUMTAX aura FR "hello"
DIDDLE peak FR aura
SAY peak
ITS OVER
```
Output:
```
hello
```

---

## Control Flow

### If / Else
Use `ONGOD` for the if, optional `NO CAP` for else, and close with `DEADASS`.

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
Output:
```
zero
```

### While Loop
Use `SKIBIDI` to start a loop and `RIZZUP` to end it.

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
```
3
2
1
```

### Truthiness
- Numbers: `0` is false; any other number is true.  
- Strings: empty is false; non-empty is true.

---

## Expressions
- Literals: integers (e.g., `123`), strings in double quotes (supports escapes like `\"` and `\n`).  
- Operands can be braincells or literals.  
- Operator precedence: `😏` and `🚡` before `💀` and `😭`.  
- Parentheses are **not** supported (keep expressions simple or split across assigns).

Examples:
```brainrot
FANUMTAX mog FR 10 😏 2 💀 5      🖕 (10*2)+5 = 25
FANUMTAX npc FR "hi" 💀 "!"       🖕 "hi!"
FANUMTAX sigma FR "ha" 😏 4       🖕 "hahaha"
```

---

## Errors (runtime vibes)
- Unknown braincell name → error  
- Copy from an unset braincell → error  
- Division by zero → error  
- Bad syntax (missing `FR`, mismatched blocks, etc.) → error

---

## Functions
Define functions with `TRALALERO` and end with `TRALALA`. Functions can take parameters and optionally return values.

```brainrot
TRALALERO greet(name)
  FANUMTAX message FR "wsg " 💀 name 💀 "!"
  RETURN message
TRALALA

LOCK IN
FANUMTAX greeting FR greet("sigma")
SAY greeting
ITS OVER
```

**Function Rules:**
- Function names follow the same rules as braincells
- Parameters are separated by commas
- `RETURN` is optional; default return is empty string `""`
- Functions must be defined before `LOCK IN`
- Functions can call other functions

**Example with multiple parameters:**
```brainrot
TRALALERO add(a, b)
  RETURN a 💀 b
TRALALA

TRALALERO multiply(x, y)
  RETURN x 😏 y
TRALALA

LOCK IN
FANUMTAX result FR add(5, 3)
SAY result
FANUMTAX product FR multiply(4, 6)
SAY product
ITS OVER
```

---

## Built-in Functions

The interpreter provides several built-in functions for system operations and file I/O:

| Normal name | Brainrot identifier (what you'll use in code) | Example call (Brainrot syntax) |
|------------|----------------------------------------------|--------------------------------|
| `module.load` | `fanum.tax` | `fanum.tax "compiler.br"` → returns a module table or the exported function set |
| `file-read all` | `edge.mewing` | `edge.mewing "src/brainrot_compiler.br"` → string |
| `file-write all` | `edge.looksmaxxing` | `edge.looksmaxxing "build/main.br", compiled_source` |
| `system.exit` | `goated.yeet` | `goated.yeet 0` |
| `system.args` | `goated.67` | `goated.67` → list of command-line arguments |
| `system.env` | `goated.bussin` | `goated.bussin "PATH"` → value of the PATH env variable (or empty string) |

**How the VM sees them**

The interpreter registers each of these names as a built-in function. When it encounters, say, `edge.mewing "foo.txt"`, it calls the native implementation that reads the file and pushes the resulting string onto the stack (or returns it in the expression result).

**With those seven primitives in place you can:**

1. Read your compiler source (`edge.mewing`).
2. Execute it (the interpreter will parse & run the Brainrot code).
3. Handle command-line arguments (`goated.67`) or environment variables (`goated.bussin`).
4. Exit cleanly (`goated.yeet`).

---

## Mini Cheatsheet
- **Assign:** `FANUMTAX <cell> FR <expr>`  
- **Copy:** `DIDDLE <dest> FR <source>`  
- **Print:** `SAY <expr>`  
- **If:** `ONGOD <expr> … NO CAP … DEADASS` (else optional)  
- **While:** `SKIBIDI <expr> … RIZZUP`  
- **Function:** `TRALALERO <name>(params) … RETURN <expr> … TRALALA`
- **Ops:** `💀 +`, `😭 -`, `😏 *`, `🚡 /`
