use std::{env, fs};

// TODO: Loops inside words are broken; the code generation doesn't differentiate different loops:
// : loop 0 > while over . swap 1 - swap end ;
// 10 loop --> works as expected
// 15 loop --> breaks here; 'multiple definitions of @loop'
// Need to find some way to differentiate loop labels in the code gen; if labels have the same issue

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(format!("Please provide a file path"));
    }

    let file_path: Vec<&str> = args[1].split("/").collect();
    let source_file = file_path[file_path.len() - 1];
    let program = fs::read_to_string(args[1].clone()).unwrap();
    println!("{}: {:?}", source_file, program);

    let mut l = rorth::lexer::lexer::new(source_file.to_string(), program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut p = rorth::parser::parser::new(source_file.to_string(), l.tokens);
    if let Err(e) = p.parse() {
        return Err(e);
    }
    let mut c = rorth::compiler::compiler::new(source_file.to_string(), p.tokens);
    if let Err(e) = c.compile() {
        return Err(e);
    }

    Ok(())
}
