use std::{env, fs};

use rorth::{compiler::compiler, lexer::lexer, parser::parser, vm::vm};

// TODO: Fix using variables in loop conditional
// TODO: Write interpreter
// TODO: Write bytecode compiler and vm

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(format!("Please provide a file path"));
    }

    let file_path: Vec<&str> = args[1].split("/").collect();
    let source_file = file_path[file_path.len() - 1];

    let program = fs::read_to_string(args[1].clone()).unwrap();

    let mut l = lexer::new(source_file.to_string(), program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut p = parser::new(source_file.to_string(), l.tokens);
    if let Err(e) = p.parse() {
        return Err(e);
    }
    let mut c = compiler::new(source_file.to_string(), p.tokens);
    if let Err(e) = c.compile() {
        return Err(e);
    }

    let mut vm = vm::new(c.bytes);
    if let Err(e) = vm.disassemble() {
        return Err(e);
    }

    Ok(())
}
