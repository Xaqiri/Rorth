fn main() -> Result<(), String> {
    let source_file = "main.rs".to_string();
    // let program =
    //     ": fib (a -- a) 1 x := 1 > while over x * := swap 1 - swap end x ;  10 fib .".to_string();

    let program = "
: double ( a -- ) dup + ;
: dec ( a b -- b a ) swap 1 - swap dbg ;
: source ( -- ) 2 4 dec ;

source"
        .to_string();

    println!("{}: {:?}", source_file, program);

    let mut l = rorth::lexer::lexer::new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut p = rorth::parser::parser::new(source_file.clone(), l.tokens);
    if let Err(e) = p.parse() {
        return Err(e);
    }
    // p.print();
    let mut c = rorth::compiler::compiler::new(source_file.clone(), p.tokens);
    if let Err(e) = c.compile() {
        return Err(e);
    }

    Ok(())
}
