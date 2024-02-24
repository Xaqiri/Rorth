use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, Token, TokenType};

#[derive(Debug)]
enum End {
    Cond,
    Loop,
}

fn write_op(file: &mut File, stack: i32, op: &str) -> i32 {
    let s = format!("\t%n{} =d {} %n{}, %n{}\n", stack - 1, op, stack - 1, stack);
    file.write(s.as_bytes()).unwrap();
    stack - 1
}

fn print_op(file: &mut File, stack: i32, source: &String, tok: Token) -> Result<i32, String> {
    match stack {
        0 => Err(format!(
            "{}:{}:{}: Invalid '.': Nothing on the stack to print",
            source, tok.col, tok.row
        )),
        _ => {
            let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", stack);
            file.write(s.as_bytes()).unwrap();
            Ok(stack)
        }
    }
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

fn over_op(file: &mut File, stack: i32, source: &str, tok: Token) -> Result<i32, String> {
    match stack {
        0 | 1 => Err(format!(
            "{}:{}:{}: Invalid 'over': Not enough values on the stack",
            source, tok.col, tok.row
        )),
        _ => {
            let s = format!("\t%n{} =d add 0, %n{}\n", stack + 1, stack - 1);
            file.write(s.as_bytes()).unwrap();
            Ok(stack + 1)
        }
    }
}

fn push_op(file: &mut File, stack: i32, value: u32) -> i32 {
    let stack = stack + 1;
    let s = format!("\t%n{} =d add 0, d_{}\n", stack, value);
    file.write(s.as_bytes()).unwrap();
    stack
}

fn comp_op(file: &mut File, stack: i32, op: &str) -> String {
    let op = match op {
        "=" => "eq",
        "!=" => "ne",
        "<=" => "le",
        "<" => "lt",
        ">=" => "ge",
        ">" => "gt",
        _ => todo!(),
    };
    // \t%n{} =d swtof %b\n
    let s = format!(
        "\t%c1 =d add 0, %n{}\n\t%c2 =d add 0, %n{}\n\t%b =w c{}d %c1, %c2\n",
        stack - 1,
        stack,
        op,
    );
    file.write(s.as_bytes()).unwrap();
    format!("\t%b =w c{}d %c1, %c2\n", op)
}

fn main() -> Result<(), String> {
    let source_file = "main.rs".to_string();
    let program = "1 1 = ? 2 2 = ? 2 . : 4 . : 0 . end".to_string();
    //     let program = "
    // 10 0 > while
    // over . swap 1 - swap
    // end"
    //     .to_string();
    println!("{}: {:?}", source_file, program);

    let mut l = new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut stack = 0;
    let mut if_stack = 0;
    let mut else_stack = 0;
    let mut loop_stack = 0;
    let mut cond_str = "".to_string();
    let mut cur_block = End::Cond;
    let mut file = File::create("../out/rorth.ssa").unwrap();
    file.write(b"export function w $main() {\n@start\n")
        .unwrap();

    for tok in &mut l.tokens {
        match tok.tok_type {
            TokenType::QMARK(_) => {
                if_stack += 1;
                cur_block = End::Cond;
                tok.tok_type = TokenType::QMARK(if_stack);
            }
            TokenType::COLON(_) => {
                else_stack += 1;
                if else_stack > if_stack {
                    return Err(format!(
                        "{}:{}:{}: Invalid ':': Can't use ':' without preceding '?'",
                        source_file, tok.col, tok.row
                    ));
                }
                tok.tok_type = TokenType::COLON(else_stack);
            }
            TokenType::WHILE(_) => {
                loop_stack += 1;
                cur_block = End::Loop;
                tok.tok_type = TokenType::WHILE(loop_stack);
            }
            TokenType::END(_) => match cur_block {
                End::Cond => tok.tok_type = TokenType::END(if_stack),
                End::Loop => tok.tok_type = TokenType::END(loop_stack),
            },
            _ => continue,
        }
    }
    for tok in l.tokens {
        match tok.tok_type {
            TokenType::PLUS => stack = write_op(&mut file, stack, "add"),
            TokenType::MINUS => stack = write_op(&mut file, stack, "sub"),
            TokenType::ASTERISK => stack = write_op(&mut file, stack, "mul"),
            TokenType::SLASH => stack = write_op(&mut file, stack, "div"),
            TokenType::EQUAL => cond_str = comp_op(&mut file, stack, "="),
            TokenType::NEQUAL => cond_str = comp_op(&mut file, stack, "!="),
            TokenType::LTE => cond_str = comp_op(&mut file, stack, "<="),
            TokenType::LT => cond_str = comp_op(&mut file, stack, "<"),
            TokenType::GTE => cond_str = comp_op(&mut file, stack, ">="),
            TokenType::GT => cond_str = comp_op(&mut file, stack, ">"),
            TokenType::INT(i) => stack = push_op(&mut file, stack, i),
            TokenType::STR(_) => stack += 1,
            TokenType::SWAP => swap_op(&mut file, stack),
            TokenType::DROP => stack -= 1,
            TokenType::NIP => {
                swap_op(&mut file, stack);
                stack -= 1;
            }
            TokenType::OVER => match over_op(&mut file, stack, &source_file, tok) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::DUP => match dup_op(&mut file, stack, &source_file, tok) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::PERIOD => match print_op(&mut file, stack, &source_file, tok) {
                Ok(s) => stack = s - 1,
                Err(e) => return Err(e),
            },
            TokenType::COMMA => match print_op(&mut file, stack, &source_file, tok) {
                Ok(s) => stack = s,
                Err(e) => return Err(e),
            },
            TokenType::QMARK(pos) => {
                let mut s = format!("\tjnz %b, @if_{}, @else_{}\n", pos, pos);
                file.write(s.as_bytes()).unwrap();
                s = format!("@if_{}\n", pos);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::COLON(pos) => {
                let s = format!("\tjmp @end_if_{}\n@else_{}\n", pos, pos);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::WHILE(pos) => {
                let mut s = format!("\tjnz %b, @loop_{}, @end_loop_{}\n", pos, pos);
                file.write(s.as_bytes()).unwrap();
                s = format!("@loop_{}\n", pos);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::END(pos) => {
                let s: String;
                match cur_block {
                    End::Cond => {
                        if if_stack == else_stack {
                            s = format!("@end_if_{}\n", pos);
                        } else {
                            s = format!("@else_{}\n@end_if_{}\n", pos, pos);
                        }
                    }
                    End::Loop => {
                        let c = format!("\t%c1 =d add 0, %n{}\n", stack - 1);
                        file.write(c.as_bytes()).unwrap();
                        file.write(cond_str.as_bytes()).unwrap();
                        s = format!(
                            "\tjnz %b, @loop_{}, @end_loop_{}\n@end_loop_{}\n",
                            pos, pos, pos
                        );
                    }
                }
                file.write(s.as_bytes()).unwrap();
            }
            _ => println!("Unhandled token: {:?}", tok),
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
