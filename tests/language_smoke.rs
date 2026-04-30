use brvm::{compiler, lexer, parser, vm};
use std::io::Cursor;

fn compile_source(source: &str) -> Vec<u8> {
    let tokens = lexer::tokenize(source, "<test>").expect("lexing should succeed");
    let program = parser::parse(tokens, "<test>").expect("parsing should succeed");
    compiler::compile(program).expect("compilation should succeed")
}

fn run_source(source: &str, stdin: &str) -> String {
    let bytecode = compile_source(source);
    let mut input = Cursor::new(stdin.as_bytes());
    let mut output = Vec::new();
    vm::execute_with_io(&bytecode, &mut input, &mut output).expect("execution should succeed");
    String::from_utf8(output).expect("vm output should be UTF-8")
}

#[test]
fn bundled_examples_compile() {
    for source in [
        include_str!("../examples/v1.brainrot"),
        include_str!("../examples/v2.brainrot"),
        include_str!("../examples/v3.brainrot"),
        include_str!("../examples/v4.brainrot"),
    ] {
        compile_source(source);
    }
}

#[test]
fn touchy_prompt_writes_prompt_before_reading_input() {
    let output = run_source(
        r#"
LOCK IN
FANUMTAX aura FR TOUCHY("name: ")
SAY "hi " 💀 aura
ITS OVER
"#,
        "Ada\n",
    );

    assert_eq!(output, "name: hi Ada\n");
}

#[test]
fn user_function_arguments_keep_source_order() {
    let output = run_source(
        r#"
TRALALERO sub(a, b)
  RETREAT a 😭 b
TRALALA

LOCK IN
SAY ring yas sub(10, 3)
ITS OVER
"#,
        "",
    );

    assert_eq!(output, "7\n");
}

#[test]
fn zero_argument_functions_use_locals() {
    let output = run_source(
        r#"
TRALALERO make_message()
  FANUMTAX message FR "ok"
  RETREAT message
TRALALA

LOCK IN
SAY ring yas make_message()
ITS OVER
"#,
        "",
    );

    assert_eq!(output, "ok\n");
}

#[test]
fn recursive_functions_can_call_themselves() {
    let output = run_source(
        r#"
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
"#,
        "",
    );

    assert_eq!(output, "120\n");
}

#[test]
fn strings_can_be_repeated_with_multiply() {
    let output = run_source(
        r#"
LOCK IN
SAY "ha" 😏 3
ITS OVER
"#,
        "",
    );

    assert_eq!(output, "hahaha\n");
}
