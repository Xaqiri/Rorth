use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, TokenType};
use rorth::stack::stack::{self, *};

fn _sim() -> Result<Stack, String> {
    let source_file = "main.rs";
    let program = "1 1 = ? yes . : no .".to_string();
    println!("{}: {:?}", source_file, program);

    let mut l = new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut stack = stack::new();
    for tok in l.tokens {
        match tok.tok_type {
            TokenType::PLUS => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) + a.unwrap_or(0.0));
            }
            TokenType::MINUS => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) - a.unwrap_or(0.0));
            }
            TokenType::ASTERISK => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) * a.unwrap_or(0.0));
            }
            TokenType::SLASH => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) / a.unwrap_or(0.0));
            }
            TokenType::PERIOD => match stack.top_type {
                CurStack::Num => {
                    let a = stack.pop().ok_or(0);
                    if let Ok(i) = a {
                        print!("{} ", i);
                    } else {
                        return Err(format!(
                            "{}:{}:{}: Invalid '.': Nothing on the stack to print",
                            source_file, tok.col, tok.row
                        ));
                    }
                }
                CurStack::String => {
                    let a = stack.pop_str().ok_or(0);
                    if let Ok(i) = a {
                        print!("{} ", i);
                    } else {
                        return Err(format!(
                            "{}:{}:{}: Invalid '.': Nothing on the stack to print",
                            source_file, tok.col, tok.row
                        ));
                    }
                }
            },
            TokenType::COMMA | TokenType::PEEK => {
                let a = stack.last().ok_or(0);
                if let Err(_) = a {
                    return Err(format!(
                        "{}:{}:{}: Invalid ',': Nothing on the stack to print",
                        source_file, tok.col, tok.row
                    ));
                }
            }
            TokenType::QMARK => {
                stack.push_if();
            }
            TokenType::COLON => {
                stack.if_stack.pop();
            }
            TokenType::EQUAL => {
                let a = stack.pop().ok_or("No value on the stack");
                let b = stack.pop().ok_or("No value on the stack");
                match a == b {
                    true => stack.push(1.0),
                    false => stack.push(0.0),
                }
            }
            TokenType::INT(i) => {
                stack.push(i as f32);
                stack.top_type = CurStack::Num;
            }
            TokenType::STR(s) => {
                stack.push_str(s);
                stack
                    .string_heap
                    .insert(stack.string_stack.last().unwrap().to_string());
                stack.top_type = CurStack::String;
            }
            TokenType::SWAP => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
            }
            TokenType::DUP => {
                let a = *stack.last().unwrap();
                stack.push(a);
            }
            TokenType::EOF => {}
        }
    }
    Ok(stack)
}

fn comp() -> Result<(), String> {
    let source_file = "main.rs";
    let program = "1 1 = ? 4 swap dup + . : 2 .".to_string();
    println!("{}: {:?}", source_file, program);

    let mut l = new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut stack = 0;
    let mut if_stack = 0;
    let mut file = File::create("../out/rorth.ssa").unwrap();
    file.write(b"export function w $main() {\n@start\n")
        .unwrap();

    for tok in l.tokens {
        match tok.tok_type {
            TokenType::PLUS => {
                let s = format!("\t%n{} =d add %n{}, %n{}\n", stack - 1, stack, stack - 1);
                file.write(s.as_bytes()).unwrap();
                stack -= 1;
            }
            TokenType::MINUS => {
                let s = format!("\t%n{} =d sub %n{}, %n{}\n", stack - 1, stack, stack - 1);
                file.write(s.as_bytes()).unwrap();
                stack -= 1;
            }
            TokenType::ASTERISK => {
                let s = format!("\t%n{} =d mul %n{}, %n{}\n", stack - 1, stack, stack - 1);
                file.write(s.as_bytes()).unwrap();
                stack -= 1;
            }
            TokenType::SLASH => {
                let s = format!("\t%n{} =d div %n{}, %n{}\n", stack - 1, stack, stack - 1);
                file.write(s.as_bytes()).unwrap();
                stack -= 1;
            }
            TokenType::PERIOD => {
                let a = match stack {
                    0 => Err(1),
                    _ => Ok(stack),
                };
                if let Ok(n) = a {
                    let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", n);
                    file.write(s.as_bytes()).unwrap();
                } else {
                    return Err(format!(
                        "{}:{}:{}: Invalid '.': Nothing on the stack to print",
                        source_file, tok.col, tok.row
                    ));
                }
                stack -= 1;
            }

            // TokenType::PERIOD => match stack.top_type {
            //     CurStack::Num => {
            //         let a = match stack {
            //             0 => Err(1),
            //             _ => Ok(stack),
            //         };
            //         if let Ok(n) = a {
            //             // let s: String;
            //             // if n == n.round() {
            //             //     s = format!("\tcall $printf(l $fmt_int, ..., d %n{})\n", stack);
            //             // } else {
            //             let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", n);
            //             // }
            //             file.write(s.as_bytes()).unwrap();
            //         } else {
            //             return Err(format!(
            //                 "{}:{}:{}: Invalid '.': Nothing on the stack to print",
            //                 source_file, tok.col, tok.row
            //             ));
            //         }
            //     }
            //     CurStack::String => {
            //         let a = stack.pop_str().ok_or(0);
            //         if let Ok(n) = a {
            //             let s = format!("\tcall $printf(l $fmt_str, ..., l $str_{})\n", n);
            //             file.write(s.as_bytes()).unwrap();
            //         } else {
            //             return Err(format!(
            //                 "{}:{}:{}: Invalid '.': Nothing on the stack to print",
            //                 source_file, tok.col, tok.row
            //             ));
            //         }
            //     }
            // },
            TokenType::COMMA | TokenType::PEEK => {
                let a = match stack {
                    0 => Err(1),
                    _ => Ok(stack),
                };
                if let Ok(n) = a {
                    let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", n);
                    file.write(s.as_bytes()).unwrap();
                } else {
                    return Err(format!(
                        "{}:{}:{}: Invalid ',': Nothing on the stack to print",
                        source_file, tok.col, tok.row
                    ));
                }
            }
            TokenType::QMARK => {
                if_stack += 1;
                let mut s = format!("\tjnz %b, @if_{}, @else_{}\n", if_stack, if_stack);
                file.write(s.as_bytes()).unwrap();
                s = format!("@if_{}\n", if_stack);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::COLON => {
                let s = format!("\tjmp @end\n@else_{}\n", if_stack);
                file.write(s.as_bytes()).unwrap();
                if_stack -= 1;
            }
            TokenType::EQUAL => {
                let s = format!(
                    "\t%b =w ceqd %n{}, %n{}\n\t%n{} =d swtof %b\n",
                    stack,
                    stack - 1,
                    stack,
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::INT(i) => {
                stack += 1;
                let s = format!("\t%n{} =d add 0, d_{}\n", stack, i);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::STR(_) => {
                stack += 1;
            }
            TokenType::SWAP => {
                let a = stack;
                let b = stack - 1;
                let s = format!("\t%t =d add 0, %n{}\n", a);
                file.write(s.as_bytes()).unwrap();
                let s = format!("\t%n{} =d add 0, %n{}\n", a, b);
                file.write(s.as_bytes()).unwrap();
                let s = format!("\t%n{} =d add 0, %t\n", b);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::DUP => {
                stack += 1;
                let s = format!("\t%n{} =d add 0, %n{}\n", stack, stack - 1);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::EOF => {}
        }
    }

    file.write(b"@end\n\tret 0\n}\n").unwrap();
    file.write(b"data $fmt_int = { b \"%.f \", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_dec = { b \"%f \", b 0 }\n").unwrap();
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
    let t = cmd.stdout;
    io::stdout().write_all(&t).unwrap();
    Ok(())
}

fn main() -> Result<(), String> {
    comp()
}
