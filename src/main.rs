use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{EndBlock, Token, TokenType};

fn write_op(file: &mut File, stack: i32, op: &str) -> i32 {
    let s = format!("\t%n{} =d {} %n{}, %n{}\n", stack - 1, op, stack - 1, stack);
    file.write(s.as_bytes()).unwrap();
    stack - 1
}

fn push_op(file: &mut File, stack: i32, value: Token) -> i32 {
    let stack = stack + 1;
    let s: String;
    match &value.tok_type {
        TokenType::INT(i) => s = format!("\t%n{} =d add 0, d_{}\n", stack, i),
        TokenType::IDENT(i) => s = format!("\t%n{} =d add 0, %{}\n", stack, i),
        _ => panic!("Invalid push target: {:?}", value),
    }
    file.write(s.as_bytes()).unwrap();
    stack
}

fn comp_op(file: &mut File, stack: i32, op: &str) -> (i32, String) {
    let stack = stack + 1;
    let op = match op {
        "=" => "eq",
        "!=" => "ne",
        "<=" => "le",
        "<" => "lt",
        ">=" => "ge",
        ">" => "gt",
        _ => panic!("Unknown op: {}", op),
    };
    let s = format!(
        "\t%b =w c{}d %n{}, %n{}\n\t%n{} =d swtof %b\n",
        op,
        stack - 2,
        stack - 1,
        stack,
    );
    file.write(s.as_bytes()).unwrap();
    (stack, s)
}

fn set_op(file: &mut File, stack: i32, var_name: &String) -> Result<i32, String> {
    let s: String = format!("\t%{} =d add 0, %n{}\n", var_name, stack);
    file.write(s.as_bytes()).unwrap();
    Ok(stack - 1)
}

fn print_op(file: &mut File, stack: i32) -> Result<i32, String> {
    let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", stack);
    file.write(s.as_bytes()).unwrap();
    Ok(stack - 1)
}

fn dup_op(file: &mut File, stack: i32, source: &String, tok: Token) -> Result<i32, String> {
    match stack {
        0 => Err(format!(
            "{}:{}:{}: Invalid 'dup': Nothing on the stack to print",
            source, tok.col, tok.row
        )),
        _ => {
            let s = format!("\t%n{} =d add 0, %n{}\n", stack + 1, stack);
            file.write(s.as_bytes()).unwrap();
            Ok(stack + 1)
        }
    }
}

fn swap_op(file: &mut File, stack: i32) {
    let a = stack;
    let b = stack - 1;
    let s = format!(
        "\t%t =d add 0, %n{}\n\t%n{} =d add 0, %n{}\n\t%n{} =d add 0, %t\n",
        a, a, b, b
    );
    file.write(s.as_bytes()).unwrap();
}

fn over_op(file: &mut File, stack: i32) -> Result<i32, String> {
    let s = format!("\t%n{} =d add 0, %n{}\n", stack + 1, stack - 1);
    file.write(s.as_bytes()).unwrap();
    Ok(stack + 1)
}

fn compile(tokens: Vec<Token>, source_file: String) -> Result<(), String> {
    let mut stack = 0;
    let mut cond_str = "".to_string();
    let mut var_stack: Vec<String> = vec![];
    let mut vars: HashSet<String> = HashSet::new();
    let mut file = File::create("../out/rorth.ssa").unwrap();
    file.write(b"export function w $main() {\n@start\n")
        .unwrap();

    for tok in tokens {
        match tok.tok_type {
            TokenType::PLUS => stack = write_op(&mut file, stack, "add"),
            TokenType::MINUS => stack = write_op(&mut file, stack, "sub"),
            TokenType::ASTERISK => stack = write_op(&mut file, stack, "mul"),
            TokenType::SLASH => stack = write_op(&mut file, stack, "div"),
            TokenType::EQUAL => (stack, cond_str) = comp_op(&mut file, stack, "="),
            TokenType::NEQUAL => (stack, cond_str) = comp_op(&mut file, stack, "!="),
            TokenType::LTE => (stack, cond_str) = comp_op(&mut file, stack, "<="),
            TokenType::LT => (stack, cond_str) = comp_op(&mut file, stack, "<"),
            TokenType::GTE => (stack, cond_str) = comp_op(&mut file, stack, ">="),
            TokenType::GT => (stack, cond_str) = comp_op(&mut file, stack, ">"),
            TokenType::INT(_) => stack = push_op(&mut file, stack, tok),
            TokenType::STR(_) => stack += 1,
            TokenType::SWAP => swap_op(&mut file, stack),
            TokenType::DROP => stack -= 1,
            TokenType::NIP => {
                swap_op(&mut file, stack);
                stack -= 1;
            }
            TokenType::OVER => match over_op(&mut file, stack) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::DUP => match dup_op(&mut file, stack, &source_file, tok) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::PERIOD => match print_op(&mut file, stack) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::COMMA => match print_op(&mut file, stack) {
                Ok(s) => stack = s + 1,
                Err(e) => return Err(e),
            },
            TokenType::SET => {
                let var = var_stack.last();
                if let Some(v) = var {
                    stack = match set_op(&mut file, stack, v) {
                        Ok(i) => i,
                        Err(e) => return Err(e),
                    };
                } else {
                    return Err("No variable to assign to".to_string());
                }
            }
            TokenType::IDENT(ref s) => {
                match vars.insert(s.to_string()) {
                    true => var_stack.push(s.to_string()),
                    false => stack = push_op(&mut file, stack, tok),
                };
            }
            TokenType::IF(pos) => {
                let s = format!(
                    "\t%b =w dtosi %n{}\n\tjnz %b, @if_{}, @else_{}\n@if_{}\n",
                    stack, pos, pos, pos
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::ELSE(pos) => {
                let s = format!("\tjmp @end_if_{}\n@else_{}\n", pos, pos);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::WHILE(op, pos) => {
                stack -= 1;
                let s = format!("\t%c_{}_{} =d add 0, %n{}\n", pos, pos, stack - 1);
                file.write(s.as_bytes()).unwrap();
                let s = format!("\t%c_{}_{} =d add 0, %n{}\n", pos, pos + 1, stack);
                file.write(s.as_bytes()).unwrap();
                let comp = match *op.to_owned() {
                    TokenType::EQUAL => "eq",
                    TokenType::NEQUAL => "ne",
                    TokenType::LTE => "le",
                    TokenType::LT => "lt",
                    TokenType::GTE => "ge",
                    TokenType::GT => "gt",
                    _ => todo!(),
                };
                let s = format!(
                    "\t%b =w c{}d %c_{}_{}, %c_{}_{}\n",
                    comp,
                    pos,
                    pos,
                    pos,
                    pos + 1
                );
                file.write(s.as_bytes()).unwrap();
                let s = format!(
                    "\tjnz %b, @loop_{}, @end_loop_{}\n@loop_{}\n",
                    pos, pos, pos
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::END(cur_block, pos) => {
                let s: String;
                match cur_block {
                    EndBlock::Cond => {
                        if pos == 0 {
                            s = format!("@else_{}\n@end_if_{}\n", pos, pos);
                        } else {
                            s = format!("@end_if_{}\n", pos);
                        }
                        file.write(s.as_bytes()).unwrap();
                    }
                    EndBlock::Loop => {
                        s = format!("\t%c_{}_{} =d sub %c_{}_{}, 1\n", pos, pos, pos, pos,);
                        file.write(s.as_bytes()).unwrap();
                        file.write(cond_str.as_bytes()).unwrap();
                        let s = format!(
                            "\tjnz %b, @loop_{}, @end_loop_{}\n@end_loop_{}\n",
                            pos, pos, pos
                        );
                        file.write(s.as_bytes()).unwrap();
                    }
                }
            }
            TokenType::EOF => break,
            _ => return Err(format!("compiler: Unhandled token: {:?}", tok)),
        }
    }

    file.write(b"@end\n\tret 0\n}\n").unwrap();
    file.write(b"data $fmt_int = { b \"%.f \", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_dec = { b \"%.10g \", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_str = { b \"%s \", b 0 }\n").unwrap();

    // for i in string_heap {
    //     let s = format!("data $str_{} = {{ b \"{}\", b 0 }}\n", i, i);
    //     file.write(s.as_bytes()).unwrap();
    // }
    Ok(())
}

fn main() -> Result<(), String> {
    let source_file = "main.rs".to_string();
    let program = "
1 x :=
10 1 > while
  over x * :=
  swap 1 - swap
end x ."
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
    p.print();
    if let Err(e) = compile(p.tokens, source_file) {
        return Err(e);
    }

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("qbe -o ../out/out.s ../out/rorth.ssa && gcc -o ../out/rorth ../out/out.s && ../out/rorth")
        .output()
        .expect("failed to execute command");
    let output = cmd.stdout;
    io::stdout().write_all(&output).unwrap();

    println!();
    Ok(())
}
