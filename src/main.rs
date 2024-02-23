// use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, TokenType};

fn main() {
    let program = "8 12 + 5 / 4 = .".to_string();
    println!("main.rs: {:?}", program);

    let mut l = new(program);
    l.lex();
    let mut stack: Vec<f32> = vec![];
    //     let mut string_stack: Vec<String> = vec![];
    //     let mut string_heap = HashSet::new();
    let mut file = File::create("../out/rorth.ssa").unwrap();
    file.write(b"export function w $main() {\n@start\n")
        .unwrap();
    for tok in l.tokens {
        match tok.tok_type {
            TokenType::PLUS => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) + a.unwrap_or(0.0));
                let s = format!(
                    "\t%n{} =d add %n{}, %n{}\n",
                    stack.len(),
                    stack.len(),
                    stack.len() + 1
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::MINUS => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) - a.unwrap_or(0.0));
                let s = format!(
                    "\t%n{} =d sub %n{}, %n{}\n",
                    stack.len(),
                    stack.len(),
                    stack.len() + 1
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::ASTERISK => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) * a.unwrap_or(0.0));
                let s = format!(
                    "\t%n{} =d mul %n{}, %n{}\n",
                    stack.len(),
                    stack.len(),
                    stack.len() + 1
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::SLASH => {
                let a = stack.pop();
                let b = stack.pop();
                stack.push(b.unwrap_or(0.0) / a.unwrap_or(0.0));
                let s = format!(
                    "\t%n{} =d div %n{}, %n{}\n",
                    stack.len(),
                    stack.len(),
                    stack.len() + 1
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::PERIOD => {
                let a = stack.pop().ok_or(0);
                match a {
                    Ok(n) => {
                        let s: String;
                        if n == n.round() {
                            s = format!(
                                "\tcall $printf(l $fmt_int, ..., d %n{})\n",
                                stack.len() + 1
                            );
                        } else {
                            s = format!(
                                "\tcall $printf(l $fmt_dec, ..., d %n{})\n",
                                stack.len() + 1
                            );
                        }
                        file.write(s.as_bytes()).unwrap();
                    }
                    Err(_) => {
                        panic!("main.rs: Invalid '.': Nothing on the stack to print")
                    }
                }
            }
            TokenType::COMMA => {
                todo!()
            }
            TokenType::INT(i) => {
                stack.push(i as f32);
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len(), i);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::EQUAL => {
                let a = stack.pop().ok_or("No value on the stack");
                let b = stack.pop().ok_or("No value on the stack");
                match a == b {
                    true => stack.push(1.0),
                    false => stack.push(0.0),
                }
                let s = format!(
                    "\t%b =w ceqd %n{}, %n{}\n\t%n{} =d swtof %b\n",
                    stack.len(),
                    stack.len() + 1,
                    stack.len()
                );

                file.write(s.as_bytes()).unwrap();
            }

            TokenType::EOF => {}
        }
        //                 ',' => {
        //                     let a = string_stack.pop().ok_or("");

        //                     match a {
        //                         Ok(n) => {
        //                             let s: String;
        //                             s = format!("\tcall $printf(l $fmt_str, ..., l $str_{})\n", n);
        //                             file.write(s.as_bytes()).unwrap();
        //                         }
        //                         Err(_) => {
        //                             panic!("main.rs: Invalid ',': Nothing on the stack to print");
        //                         }
        //                     }
        //                 }
    }

    file.write(b"\tret 0\n}\n").unwrap();
    file.write(b"data $fmt_int = { b \"%.f\\n\", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_dec = { b \"%f\\n\", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_str = { b \"%s\\n\", b 0 }\n")
        .unwrap();

    // for i in string_heap {
    //     let s = format!("data $str_{} = {{ b \"{}\", b 0 }}\n", i, i);
    //     file.write(s.as_bytes()).unwrap();
    // }

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("cd ../out/ && qbe -o out.s rorth.ssa && gcc -o rorth out.s && ./rorth")
        .output()
        .expect("failed to execute command");
    let t = cmd.stdout;
    io::stdout().write_all(&t).unwrap();
}
