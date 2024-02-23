use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, TokenType};

enum CurStack {
    Num,
    String,
}

fn main() -> Result<(), String> {
    let source_file = "main.rs";
    let program = "1 2 swap dup + . .".to_string();
    println!("{}: {:?}", source_file, program);

    let mut l = new(program);
    if let Err(e) = l.lex() {
        return Err(e);
    }
    let mut stack: Vec<f32> = vec![];
    let mut string_stack: Vec<String> = vec![];
    let mut string_heap: HashSet<String> = HashSet::new();
    let mut top_type = CurStack::Num;
    let mut if_stack = vec![];
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
            TokenType::PERIOD => match top_type {
                CurStack::Num => {
                    let a = stack.pop().ok_or(0);
                    if let Ok(n) = a {
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
                    } else {
                        return Err(format!(
                            "{}:{}:{}: Invalid '.': Nothing on the stack to print",
                            source_file, tok.col, tok.row
                        ));
                    }
                }
                CurStack::String => {
                    let a = string_stack.pop().ok_or(0);
                    if let Ok(n) = a {
                        let s = format!("\tcall $printf(l $fmt_str, ..., l $str_{})\n", n);
                        file.write(s.as_bytes()).unwrap();
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
            TokenType::QMARK => {
                if_stack.push(1);
                let mut s = format!(
                    "\tjnz %b{}, @if_{}, @else_{}\n",
                    stack.len(),
                    if_stack.len(),
                    if_stack.len()
                );
                file.write(s.as_bytes()).unwrap();
                s = format!("@if_{}\n", if_stack.len());
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::COLON => {
                let s = format!("\tjmp @end\n@else_{}\n", if_stack.len());
                file.write(s.as_bytes()).unwrap();
                if_stack.pop();
            }
            TokenType::EQUAL => {
                let a = stack.pop().ok_or("No value on the stack");
                let b = stack.pop().ok_or("No value on the stack");
                match a == b {
                    true => stack.push(1.0),
                    false => stack.push(0.0),
                }
                let s = format!(
                    "\t%b{} =w ceqd %n{}, %n{}\n\t%n{} =d swtof %b{}\n",
                    stack.len(),
                    stack.len(),
                    stack.len() + 1,
                    stack.len(),
                    stack.len()
                );
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::INT(i) => {
                stack.push(i as f32);
                top_type = CurStack::Num;
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len(), i);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::STR(s) => {
                string_stack.push(s);
                string_heap.insert(string_stack.last().unwrap().to_string());
                top_type = CurStack::String;
            }
            TokenType::SWAP => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len() - 1, a);
                file.write(s.as_bytes()).unwrap();
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len(), b);
                file.write(s.as_bytes()).unwrap();
            }
            TokenType::DUP => {
                let a = *stack.last().unwrap();
                stack.push(a);
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len(), a);
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

    for i in string_heap {
        let s = format!("data $str_{} = {{ b \"{}\", b 0 }}\n", i, i);
        file.write(s.as_bytes()).unwrap();
    }

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("qbe -o ../out/out.s ../out/rorth.ssa && gcc -o ../out/rorth ../out/out.s && ../out/rorth")
        .output()
        .expect("failed to execute command");
    let t = cmd.stdout;
    io::stdout().write_all(&t).unwrap();
    Ok(())
}
