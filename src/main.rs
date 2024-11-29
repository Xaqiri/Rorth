use std::{env, fs};

use rorth::{lexer::lexer, parser::parser, qbe_backend};
// use rorth::{compiler::compiler, vm::vm}

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
    let file_type = source_file.split(".").last();
    if let Some("rorth") = file_type {
        let program = fs::read_to_string(args[1].clone()).unwrap();

        let mut l = lexer::new(source_file.to_string(), program);
        if let Err(e) = l.lex() {
            return Err(e);
        }
        let mut p = parser::new(source_file.to_string(), l.tokens);
        if let Err(e) = p.parse() {
            return Err(e);
        }
        let mut c = qbe_backend::qbe_backend::new(source_file.to_string(), p.tokens);
        if let Err(e) = c.compile() {
            return Err(e);
        }
        // let mut c = compiler::new(source_file.to_string(), p.tokens);
        // if let Err(e) = c.compile() {
        //     return Err(e);
        // }
        //
        // let mut vm = vm::new(c.bytes, c.const_pool);
        // if let Err(e) = vm.interpret() {
        //     return Err(e);
        // }
    } else if let Some("rvm") = file_type {
        return Err("Bytecode interpreter not yet implemented".to_string());
    } else {
        return Err(format!("Invalid file type: {:?}", file_type));
    }
    Ok(())
}
