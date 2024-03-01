pub mod compiler {
    use std::{
        collections::{HashMap, HashSet},
        fs::File,
        io::{self, Write},
        process::Command,
    };

    use crate::lexer::lexer::{EndBlock, Token, TokenType};

    pub struct Compiler {
        source: String,
        tokens: Vec<Token>,
        file: File,
        stack: i32,
        pos: usize,
        peek: usize,
        words: HashMap<String, i32>,
    }

    pub fn new(source: String, tokens: Vec<Token>) -> Compiler {
        let file = File::create("../out/rorth.ssa").unwrap();
        Compiler {
            source,
            tokens,
            file,
            stack: 0,
            pos: 0,
            peek: 1,
            words: HashMap::new(),
        }
    }

    impl Compiler {
        fn write_op(&mut self, op: &str) -> i32 {
            let s = format!(
                "\t%n{} =d {} %n{}, %n{}\n",
                self.stack - 1,
                op,
                self.stack - 1,
                self.stack
            );
            self.file.write(s.as_bytes()).unwrap();
            self.stack - 1
        }

        fn push_op(&mut self, value: &Token) -> i32 {
            let stack = self.stack + 1;
            let s: String;
            match &value.tok_type {
                TokenType::INT(i) => s = format!("\t%n{} =d add 0, d_{}\n", stack, i),
                TokenType::IDENT(i) => s = format!("\t%n{} =d add 0, %{}\n", stack, i),
                _ => panic!("Invalid push target: {:?}", value),
            }
            self.file.write(s.as_bytes()).unwrap();
            stack
        }

        fn comp_op(&mut self, op: &str) -> (i32, String) {
            let stack = self.stack + 1;
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
            self.file.write(s.as_bytes()).unwrap();
            (stack, s)
        }

        fn set_op(&mut self, var_name: &String) -> Result<i32, String> {
            let s: String = format!("\t%{} =d add 0, %n{}\n", var_name, self.stack);
            self.file.write(s.as_bytes()).unwrap();
            Ok(self.stack - 1)
        }

        fn print_op(&mut self) -> Result<i32, String> {
            let s = format!("\tcall $printf(l $fmt_dec, ..., d %n{})\n", self.stack);
            self.file.write(s.as_bytes()).unwrap();
            Ok(self.stack - 1)
        }

        fn dup_op(&mut self, tok: &Token) -> Result<i32, String> {
            match self.stack {
                0 => Err(format!(
                    "{}:{}:{}: Invalid 'dup': Nothing on the stack to print",
                    self.source, tok.col, tok.row
                )),
                _ => {
                    let s = format!("\t%n{} =d add 0, %n{}\n", self.stack + 1, self.stack);
                    self.file.write(s.as_bytes()).unwrap();
                    Ok(self.stack + 1)
                }
            }
        }

        fn swap_op(&mut self) {
            let a = self.stack;
            let b = self.stack - 1;
            let s = format!(
                "\t%t =d add 0, %n{}\n\t%n{} =d add 0, %n{}\n\t%n{} =d add 0, %t\n",
                a, a, b, b
            );
            self.file.write(s.as_bytes()).unwrap();
        }

        fn over_op(&mut self) -> Result<i32, String> {
            let s = format!("\t%n{} =d add 0, %n{}\n", self.stack + 1, self.stack - 1);
            self.file.write(s.as_bytes()).unwrap();
            Ok(self.stack + 1)
        }

        fn rot_op(&mut self) {
            let a = self.stack;
            let b = self.stack - 1;
            let c = self.stack - 2;
            let s = format!(
                "\t%t =d add 0, %n{}\n\t%n{} =d add 0, %n{}\n\t%n{} =d add 0, %n{}\n\t%n{} =d add 0, %t\n",
                c, c, b, b, a, a
            );
            self.file.write(s.as_bytes()).unwrap();
        }

        fn new_word_op(&mut self) -> Result<u32, String> {
            self.advance_token();
            let cur_token = self.tokens[self.pos].clone();
            let func_name = match &cur_token.tok_type {
                TokenType::IDENT(s) => s,
                _ => return Err(format!("New word error: Invalid name")),
            };
            self.words.insert(func_name.clone(), 0);
            if self.peek().tok_type == TokenType::LPAREN {
                self.advance_token()
            }
            let mut arg_list: Vec<String> = vec![];
            self.advance_token();
            let mut cur_token = self.tokens[self.pos].clone();
            while cur_token.tok_type != TokenType::EM {
                let arg = match cur_token.tok_type {
                    TokenType::IDENT(s) => s,
                    _ => return Err(format!("New word error: Found {:?}", cur_token.tok_type)),
                };
                arg_list.push(arg);
                self.stack += 1;
                self.advance_token();
                cur_token = self.tokens[self.pos].clone();
            }
            while cur_token.tok_type != TokenType::RPAREN {
                self.advance_token();
                cur_token = self.tokens[self.pos].clone();
            }
            self.advance_token();

            self.words
                .entry(func_name.to_string())
                .and_modify(|e| *e = self.stack);
            let mut arg_str = format!("");
            for i in 1..=self.stack {
                if i < self.stack {
                    arg_str.push_str(&format!("d %n{}, ", i))
                } else {
                    arg_str.push_str(&format!("d %n{}", i))
                }
            }
            let mut s = format!("function d ${}({}) {{\n@start\n", func_name, arg_str);
            self.file.write(s.as_bytes()).unwrap();
            let res = self.parse_function_body(TokenType::SEMICOLON);

            s = format!("@end\n\tret %n{}\n}}\n", self.stack);
            self.file.write(s.as_bytes()).unwrap();
            self.stack = 0;
            res
        }

        fn advance_token(&mut self) {
            self.pos += 1;
            self.peek += 1;
        }

        fn peek(&self) -> &Token {
            &self.tokens[self.peek]
        }

        fn parse_function_body(&mut self, end: TokenType) -> Result<u32, String> {
            let mut cond_str = "".to_string();
            let mut var_stack: Vec<String> = vec![];
            let mut vars: HashSet<String> = HashSet::new();

            while self.tokens[self.pos].tok_type != end {
                let tok = self.tokens[self.pos].clone();
                match &tok.tok_type {
                    TokenType::PLUS => self.stack = self.write_op("add"),
                    TokenType::MINUS => self.stack = self.write_op("sub"),
                    TokenType::ASTERISK => self.stack = self.write_op("mul"),
                    TokenType::SLASH => self.stack = self.write_op("div"),
                    TokenType::EQUAL => (self.stack, cond_str) = self.comp_op("="),
                    TokenType::NEQUAL => (self.stack, cond_str) = self.comp_op("!="),
                    TokenType::LTE => (self.stack, cond_str) = self.comp_op("<="),
                    TokenType::LT => (self.stack, cond_str) = self.comp_op("<"),
                    TokenType::GTE => (self.stack, cond_str) = self.comp_op(">="),
                    TokenType::GT => (self.stack, cond_str) = self.comp_op(">"),
                    TokenType::INT(_) => self.stack = self.push_op(&tok),
                    TokenType::STR(_) => self.stack += 1,
                    TokenType::SWAP => self.swap_op(),
                    TokenType::DROP => self.stack -= 1,
                    TokenType::NIP => {
                        self.swap_op();
                        self.stack -= 1;
                    }
                    TokenType::ROT => self.rot_op(),
                    TokenType::OVER => match self.over_op() {
                        Ok(s) => self.stack = s,
                        Err(e) => return Err(e),
                    },
                    TokenType::DUP => match self.dup_op(&tok) {
                        Ok(s) => self.stack = s,
                        Err(e) => return Err(e),
                    },
                    TokenType::PERIOD => match self.print_op() {
                        Ok(s) => self.stack = s,
                        Err(e) => return Err(e),
                    },
                    TokenType::COMMA => match self.print_op() {
                        Ok(s) => self.stack = s + 1,
                        Err(e) => return Err(e),
                    },
                    TokenType::SET => {
                        let var = var_stack.last();
                        if let Some(v) = var {
                            self.stack = match self.set_op(v) {
                                Ok(i) => i,
                                Err(e) => return Err(e),
                            };
                        } else {
                            return Err("No variable to assign to".to_string());
                        }
                    }
                    TokenType::IDENT(ref s) => {
                        if self.words.contains_key(s) {
                            let mut arg_str = format!("");
                            for i in 1..=self.words[s] {
                                if i < self.stack {
                                    arg_str.push_str(&format!("d %n{}, ", i))
                                } else {
                                    arg_str.push_str(&format!("d %n{}", i))
                                }
                            }

                            let s = format!("\t%n{} =d call ${}({})\n", self.stack, s, arg_str);
                            self.file.write(s.as_bytes()).unwrap();
                        } else {
                            match vars.insert(s.to_string()) {
                                true => {
                                    if self.peek().tok_type != TokenType::SET {
                                        return Err(format!("Invalid: {:?} undefined", tok));
                                    }
                                    var_stack.push(s.to_string())
                                }
                                false => self.stack = self.push_op(&tok),
                            };
                        }
                    }
                    TokenType::IF(pos) => {
                        let s = format!(
                            "\t%b =w dtosi %n{}\n\tjnz %b, @if_{}, @else_{}\n@if_{}\n",
                            self.stack, pos, pos, pos
                        );
                        self.file.write(s.as_bytes()).unwrap();
                    }
                    TokenType::ELSE(pos) => {
                        let s = format!("\tjmp @end_if_{}\n@else_{}\n", pos, pos);
                        self.file.write(s.as_bytes()).unwrap();
                    }
                    TokenType::WHILE(op, pos) => {
                        self.stack -= 1;
                        let s = format!("\t%c_{}_{} =d add 0, %n{}\n", pos, pos, self.stack - 1);
                        self.file.write(s.as_bytes()).unwrap();
                        let s = format!("\t%c_{}_{} =d add 0, %n{}\n", pos, pos + 1, self.stack);
                        self.file.write(s.as_bytes()).unwrap();
                        let comp = match *op.to_owned() {
                            TokenType::EQUAL => "eq",
                            TokenType::NEQUAL => "ne",
                            TokenType::LTE => "le",
                            TokenType::LT => "lt",
                            TokenType::GTE => "ge",
                            TokenType::GT => "gt",
                            _ => {
                                return Err(format!(
                                    "compiler: Error handling comparison: {:?}",
                                    tok
                                ))
                            }
                        };
                        let s = format!(
                            "\t%b =w c{}d %c_{}_{}, %c_{}_{}\n",
                            comp,
                            pos,
                            pos,
                            pos,
                            pos + 1
                        );
                        self.file.write(s.as_bytes()).unwrap();
                        let s = format!(
                            "\tjnz %b, @loop_{}, @end_loop_{}\n@loop_{}\n",
                            pos, pos, pos
                        );
                        self.file.write(s.as_bytes()).unwrap();
                    }
                    TokenType::END(cur_block, pos) => {
                        let s: String;
                        match cur_block {
                            EndBlock::Cond => {
                                if *pos == 0 {
                                    s = format!("@else_{}\n@end_if_{}\n", pos, pos);
                                } else {
                                    s = format!("@end_if_{}\n", pos);
                                }
                                self.file.write(s.as_bytes()).unwrap();
                            }
                            EndBlock::Loop => {
                                s = format!("\t%c_{}_{} =d sub %c_{}_{}, 1\n", pos, pos, pos, pos,);
                                self.file.write(s.as_bytes()).unwrap();
                                self.file.write(cond_str.as_bytes()).unwrap();
                                let s = format!(
                                    "\tjnz %b, @loop_{}, @end_loop_{}\n@end_loop_{}\n",
                                    pos, pos, pos
                                );
                                self.file.write(s.as_bytes()).unwrap();
                            }
                        }
                    }
                    TokenType::COLON => (),
                    TokenType::SEMICOLON => self.stack = 0,
                    TokenType::LPAREN => (),
                    TokenType::RPAREN => (),
                    TokenType::EM => (),
                    TokenType::EOF => (),
                    _ => return Err(format!("compiler: Unhandled token: {:?}", tok)),
                }
                self.advance_token();
            }
            Ok(0)
        }

        fn parse_words(&mut self) -> Result<u32, String> {
            if self.tokens[self.pos].tok_type == TokenType::COLON {
                let res = self.new_word_op();
                if let Err(e) = res {
                    return Err(e);
                } else {
                    self.advance_token();
                    self.parse_words()
                }
            } else {
                return Ok(0);
            }
        }

        pub fn compile(&mut self) -> Result<u32, String> {
            let res = self.parse_words();
            if let Err(e) = res {
                return Err(e);
            }

            self.file
                .write(b"export function w $main() {\n@start\n")
                .unwrap();

            let res = self.parse_function_body(TokenType::EOF);

            self.file.write(b"@end\n\tret 0\n}\n").unwrap();
            self.file
                .write(b"data $fmt_int = { b \"%.f \", b 0 }\n")
                .unwrap();
            self.file
                .write(b"data $fmt_dec = { b \"%.10g \", b 0 }\n")
                .unwrap();
            self.file
                .write(b"data $fmt_str = { b \"%s \", b 0 }\n")
                .unwrap();

            let cmd = Command::new("sh")
        .arg("-c")
        .arg("qbe -o ../out/out.s ../out/rorth.ssa && gcc -o ../out/rorth ../out/out.s && ../out/rorth")
        .output()
        .expect("failed to execute command");
            let output = cmd.stdout;
            io::stdout().write_all(&output).unwrap();
            let errout = cmd.stderr;
            io::stdout().write_all(&errout).unwrap();
            println!();

            res
        }
    }
}
