mod lexer;
mod parser;
mod compiler;
mod vm;
mod value;
mod error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "brvm")]
#[command(about = "Brainrot v4 Compiler and Virtual Machine")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compile {
        input: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    Exec {
        input: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Compile { input, output } => {
            let output = output.unwrap_or_else(|| {
                // If no output specified, use same directory with .brbc extension
                let parent = std::path::Path::new(&input).parent().unwrap_or(std::path::Path::new("."));
                let stem = std::path::Path::new(&input)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                parent.join(format!("{}.brbc", stem)).to_string_lossy().to_string()
            });
            
            if let Err(e) = compile_file(&input, &output) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        Commands::Exec { input } => {
            if let Err(e) = execute_file(&input) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

fn compile_file(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(input)
        .map_err(|_| error::CompileError::new(input, 0, 0, "failed to read file"))?;
    
    let tokens = lexer::tokenize(&source, input)?;
    let ast = parser::parse(tokens, input)?;
    let bytecode = compiler::compile(ast)
        .map_err(|e| error::CompileError::new(input, 0, 0, &e))?;
    
    std::fs::write(output, bytecode)
        .map_err(|_| error::CompileError::new(output, 0, 0, "failed to write bytecode"))?;
    
    Ok(())
}

fn execute_file(input: &str) -> Result<(), vm::RuntimeError> {
    let bytecode = std::fs::read(input)
        .map_err(|_| vm::RuntimeError::new("failed to read bytecode file"))?;
    
    vm::execute(&bytecode)?;
    
    Ok(())
}

