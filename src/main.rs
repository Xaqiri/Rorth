// use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, TokenType};

fn main() -> Result<(), String> {
    let source_file = "main.rs";
    let program = "1 2 + 3 * 6 - 5 + 12 + 4 / , 6 , = . ".to_string();
    println!("{}: {:?}", source_file, program);

    let mut l = new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
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
                if let Ok(n) = a {
                    let s: String;
                    if n == n.round() {
                        s = format!("\tcall $printf(l $fmt_int, ..., d %n{})\n", stack.len() + 1);
                    } else {
                        s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", stack.len() + 1);
                    }
                    file.write(s.as_bytes()).unwrap();
                } else {
                    return Err(format!(
                        "{}:{}:{}: Invalid '.': Nothing on the stack to print",
                        source_file, tok.col, tok.row
                    ));
                }
            }
            TokenType::COMMA => {
                let a = stack.last().ok_or(0);
                if let Ok(&n) = a {
                    let s: String;
                    if n == n.round() {
                        s = format!("\tcall $printf(l $fmt_int, ..., d %n{})\n", stack.len());
                    } else {
                        s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", stack.len());
                    }
                    file.write(s.as_bytes()).unwrap();
                } else {
                    return Err(format!(
                        "{}:{}:{}: Invalid ',': Nothing on the stack to print",
                        source_file, tok.col, tok.row
                    ));
                }
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
        .arg("qbe -o ../out/out.s ../out/rorth.ssa && gcc -o ../out/rorth ../out/out.s && ../out/rorth")
        .output()
        .expect("failed to execute command");
    let t = cmd.stdout;
    io::stdout().write_all(&t).unwrap();
    Ok(())
}
