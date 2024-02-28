fn main() -> Result<(), String> {
    let source_file = "main.rs".to_string();
    let program = ": inc (a -- a) 1 + ;".to_string();
    // let program = "1 1 + .".to_string();

    println!("{}: {:?}", source_file, program);

    let mut l = rorth::lexer::lexer::new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut p = rorth::parser::parser::new(source_file.clone(), l.tokens);
    if let Err(e) = p.parse() {
        return Err(e);
    }
    p.print();
    let mut c = rorth::compiler::compiler::new(source_file.clone(), p.tokens);
    if let Err(e) = c.compile() {
        return Err(e);
    }

    Ok(())
}
