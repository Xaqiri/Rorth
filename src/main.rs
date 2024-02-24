use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

use rorth::lexer::lexer::{new, Token, TokenType};

fn write_op(file: &mut File, stack: i32, op: &str) -> i32 {
    let s = format!("\t%n{} =d {} %n{}, %n{}\n", stack - 1, op, stack, stack - 1);
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
    let s = format!("\t%t =d add 0, %n{}\n", a);
    file.write(s.as_bytes()).unwrap();
    let s = format!("\t%n{} =d add 0, %n{}\n", a, b);
    file.write(s.as_bytes()).unwrap();
    let s = format!("\t%n{} =d add 0, %t\n", b);
    file.write(s.as_bytes()).unwrap();
}

fn push_op(file: &mut File, stack: i32, value: u32) -> i32 {
    let stack = stack + 1;
    let s = format!("\t%n{} =d add 0, d_{}\n", stack, value);
    file.write(s.as_bytes()).unwrap();
    stack
}

fn comp_op(file: &mut File, stack: i32, op: &str) {
    let op = match op {
        "=" => "eq",
        "!=" => "ne",
        "<=" => "le",
        "<" => "lt",
        ">=" => "ge",
        ">" => "gt",
        _ => todo!(),
    };
    let s = format!(
        "\t%b =w c{}d %n{}, %n{}\n\t%n{} =d swtof %b\n",
        op,
        stack,
        stack - 1,
        stack,
    );
    file.write(s.as_bytes()).unwrap();
}

fn main() -> Result<(), String> {
    let source_file = "main.rs".to_string();
    let program = "1 1 = ? dup + . : 0 .".to_string();
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
            TokenType::PLUS => stack = write_op(&mut file, stack, "add"),
            TokenType::MINUS => stack = write_op(&mut file, stack, "sub"),
            TokenType::ASTERISK => stack = write_op(&mut file, stack, "mul"),
            TokenType::SLASH => stack = write_op(&mut file, stack, "div"),
            TokenType::EQUAL => comp_op(&mut file, stack, "="),
            TokenType::NEQUAL => comp_op(&mut file, stack, "!="),
            TokenType::LTE => comp_op(&mut file, stack, "<="),
            TokenType::LT => comp_op(&mut file, stack, "<"),
            TokenType::GTE => comp_op(&mut file, stack, ">="),
            TokenType::GT => comp_op(&mut file, stack, ">"),
            TokenType::INT(i) => stack = push_op(&mut file, stack, i),
            TokenType::STR(_) => stack += 1,
            TokenType::SWAP => swap_op(&mut file, stack),
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
            _ => println!("Unhandled token: {:?}", tok),
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

    println!();
    Ok(())
}
