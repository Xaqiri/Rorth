use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    let program = "wow hi hello , , ,";
    println!("main.rs: {}", program);
    let mut num = "".to_string();
    let mut str = "".to_string();
    let mut stack: Vec<f32> = vec![];
    let mut string_stack: Vec<String> = vec![];
    let mut string_heap = HashSet::new();
    let mut file = File::create("../out/rorth.ssa").unwrap();
    file.write(b"export function w $main() {\n@start\n")
        .unwrap();
    for i in program.chars() {
        if i.is_whitespace() {
            if num == "" && str == "" {
                continue;
            } else if num != "" {
                let x = num.parse::<f32>().unwrap();
                stack.push(x);
                let s = format!("\t%n{} =d add 0, d_{}\n", stack.len(), x);
                file.write(s.as_bytes()).unwrap();
                num = "".to_string();
                continue;
            } else if str != "" {
                string_stack.push(str.clone());
                string_heap.insert(str);

                str = "".to_string();
                continue;
            }
        }
        if i.is_digit(10) {
            num.push(i);
        } else if i.is_alphabetic() {
            str.push(i);
        } else {
            match i {
                '+' => {
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
                '-' => {
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
                '*' => {
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
                '/' => {
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
                '=' => {
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
                '.' => {
                    let a = stack.pop().ok_or(0);
                    match a {
                        Ok(n) => {
                            let s: String;
                            println!("main.rs: {}", n);
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
                ',' => {
                    let a = string_stack.pop().ok_or("");

                    match a {
                        Ok(n) => {
                            let s: String;
                            s = format!("\tcall $printf(l $fmt_str, ..., l $str_{})\n", n);
                            file.write(s.as_bytes()).unwrap();
                        }
                        Err(_) => {
                            panic!("main.rs: Invalid ',': Nothing on the stack to print");
                        }
                    }
                }
                _ => panic!("main.rs: Unknown value {}", i),
            }
        }
    }
    file.write(b"\tret 0\n}\n").unwrap();
    file.write(b"data $fmt_int = { b \"%.f\\n\", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_dec = { b \"%f\\n\", b 0 }\n")
        .unwrap();
    file.write(b"data $fmt_str = { b \"%s\\n\", b 0 }\n")
        .unwrap();

    for i in string_heap {
        let s = format!("data $str_{} = {{ b \"{}\", b 0 }}\n", i, i);
        file.write(s.as_bytes()).unwrap();
    }

    let cmd = Command::new("sh")
        .arg("-c")
        .arg("cd ../out/ && qbe -o out.s rorth.ssa && gcc -o rorth out.s && ./rorth")
        .output()
        .expect("failed to execute command");
    let t = cmd.stdout;
    io::stdout().write_all(&t).unwrap();
}
