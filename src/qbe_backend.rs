pub mod qbe_backend {
    use std::{
        collections::{HashMap, HashSet},
        fs::File,
        io::{self, Write},
        process::Command,
    };

    use crate::lexer::lexer::{EndBlock, Token, TokenType};

    enum Peek {
        Stack,
        Word(String, usize),
    }

    type StackPointer = i32;
    pub struct Compiler {
        // Name of the source file
        source: String,
        tokens: Vec<Token>,
        output_file: File,
        // Number representing the size of the stack
        stack: StackPointer,
        pos: usize,
        peek: usize,
        strings: HashSet<String>,
        // HashMap to keep track of strings on the stack
        string_stack: HashMap<StackPointer, String>,
        // HashMap in the form "word name": [list of operations associated with the word]
        words: HashMap<String, Vec<Token>>,
        var_stack: Vec<String>,
        // Set containing the names of all variables declared in the program
        vars: HashSet<String>,
        // Name of the word the compiler is currently looking at; prepended to variable names in the
        // generated qbe code to allow variable scoping
        // : word 1 x := ; => x becomes %s_word_x
        // 1 x := => x becomes %s_x
        cur_word: String,
        if_stack: i32,
        else_stack: i32,
        if_end_stack: i32,
        loop_stack: i32,
        loop_end_stack: i32,
    }

    pub fn new(source: String, tokens: Vec<Token>) -> Compiler {
        let file = File::create("../out/rorth.ssa").unwrap();
        Compiler {
            source,
            tokens,
            output_file: file,
            stack: 0,
            pos: 0,
            peek: 1,
            strings: HashSet::new(),
            string_stack: HashMap::new(),
            words: HashMap::new(),
            var_stack: vec![],
            vars: HashSet::new(),
            cur_word: "main".to_string(),
            if_stack: 0,
            else_stack: 0,
            if_end_stack: 0,
            loop_stack: 0,
            loop_end_stack: 0,
        }
    }

    impl Compiler {
        fn format_err(&self, tok: &Token, message: String) -> Result<i32, String> {
            Err(format!(
                "{}:{}:{}: {}",
                self.source, tok.row, tok.col, message
            ))
        }

        fn advance_token(&mut self) {
            self.pos += 1;
            self.peek += 1;
        }

        fn write_op(&mut self, op: &str) -> Result<i32, String> {
            if let TokenType::STR(_) = self.tokens[self.pos - 1].tok_type {
                return self.format_err(
                    &self.tokens[self.pos - 1],
                    format!("Invalid types for {}; can't use {} with strings", op, op),
                );
            }
            if let TokenType::STR(_) = self.tokens[self.pos - 2].tok_type {
                return self.format_err(
                    &self.tokens[self.pos - 2],
                    format!("Invalid types for {}; can't use {} with strings", op, op),
                );
            }
            let s = format!(
                "\t%s_main_{} =d {} %s_main_{}, %s_main_{}\n",
                self.stack - 1,
                op,
                self.stack - 1,
                self.stack
            );
            self.output_file.write(s.as_bytes()).unwrap();
            Ok(self.stack - 1)
        }

        fn push_op(&mut self, value: &Token) -> i32 {
            let stack = self.stack + 1;
            let s: String;
            match &value.tok_type {
                TokenType::INT(i) => s = format!("\t%s_main_{} =d add 0, d_{}\n", stack, i),
                TokenType::IDENT(var) => s = format!("\t%s_main_{} =d add 0, %s_{}\n", stack, var),
                _ => panic!("Invalid push target: {:?}", value),
            }
            self.output_file.write(s.as_bytes()).unwrap();
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
                "\t%b =w c{}d %s_main_{}, %s_main_{}\n\t%s_main_{} =d swtof %b\n",
                op,
                stack - 2,
                stack - 1,
                stack,
            );
            self.output_file.write(s.as_bytes()).unwrap();
            (stack, s)
        }

        fn set_op(&mut self, var_name: String, tok: Token) -> Result<i32, String> {
            if self.stack < 1 {
                return self.format_err(
                    &tok,
                    format!("Invalid {:?}: Not enough values on the stack", tok),
                );
            }

            let s = if self.string_stack.contains_key(&self.stack) {
                format!(
                    "\t%{} =l add 0, ${}\n",
                    var_name, self.string_stack[&self.stack]
                )
            } else {
                format!("\t%{} =d add 0, %s_main_{}\n", var_name, self.stack)
            };
            self.output_file.write(s.as_bytes()).unwrap();
            Ok(self.stack - 1)
        }

        fn print_op(&mut self, tok: &Token) -> Result<i32, String> {
            if self.stack == 0 {
                return self.format_err(&tok, "Nothing on the stack to print".to_string());
            }
            let s: String = match &tok.tok_type {
                TokenType::PERIOD => {
                    if self.string_stack.contains_key(&self.stack) {
                     let s = format!("\tcall $printf(l $fmt_str, ..., l ${})\n", self.string_stack.get(&self.stack).unwrap());
                        self.string_stack.remove(&self.stack);
                        s
                    } else {
                        format!("\tcall $printf(l $fmt_dec, ..., d %s_main_{})\n",self.stack)
                        }
                    },
                TokenType::CHAR => format!(
                    "\t%s_main_{}_w =w dtosi %s_main_{}\n\tcall $printf(l $fmt_char, ..., w %s_main_{}_w)\n",
                    self.stack, self.stack, self.stack
                ),
                TokenType::PRINT => format!("\tcall $printf(l $nl)\n"),
                _ => {
                    return self.format_err(tok, format!("Invalid target: {:?} not printable", tok))
                }
            };
            self.output_file.write(s.as_bytes()).unwrap();
            Ok(self.stack - 1)
        }

        fn dbg_op(&mut self) {
            println!("Stack size: {}", self.stack);
            println!("Strings: {:?}", self.string_stack);
            let s = format!("\tcall $puts(w 0)\n");
            self.output_file.write(s.as_bytes()).unwrap();
            let s = format!("\tcall $printf(l $fmt_str, ..., l $dbg)\n",);
            self.output_file.write(s.as_bytes()).unwrap();
            for i in 1..=self.stack {
                let s = if self.string_stack.contains_key(&i) {
                    format!(
                        "\tcall $printf(l $fmt_str, ..., l ${})\n",
                        self.string_stack[&i]
                    )
                } else {
                    format!("\tcall $printf(l $fmt_dec, ..., d %s_main_{})\n", i)
                };
                self.output_file.write(s.as_bytes()).unwrap();
            }
            let s = format!("\tcall $puts(w 0)\n");
            self.output_file.write(s.as_bytes()).unwrap();
        }

        fn new_word_op(&mut self) -> Result<i32, String> {
            self.advance_token();
            let cur_token = self.tokens[self.pos].clone();
            let word_name = match &cur_token.tok_type {
                TokenType::IDENT(s) => s,
                _ => {
                    return self.format_err(&cur_token, "New word error: Invalid name".to_string())
                }
            };
            self.words.insert(word_name.clone(), vec![]);
            self.cur_word = word_name.clone();

            if self.peek(Peek::Stack).tok_type == TokenType::LPAREN {
                let mut cur_token = self.tokens[self.pos].clone();
                while cur_token.tok_type != TokenType::RPAREN {
                    self.advance_token();
                    cur_token = self.tokens[self.pos].clone();
                }
                self.advance_token();
            } else {
                self.advance_token();
            }

            let mut cur_token = self.tokens[self.pos].clone();
            while cur_token.tok_type != TokenType::SEMICOLON {
                if let TokenType::IDENT(s) = &cur_token.tok_type {
                    if !self.words.contains_key(s) {
                        cur_token.tok_type = TokenType::IDENT(format!("{}_{}", self.cur_word, s));
                    }
                }
                self.words.get_mut(word_name).unwrap().push(cur_token);
                self.advance_token();
                cur_token = self.tokens[self.pos].clone();
            }
            self.words.get_mut(word_name).unwrap().push(Token {
                row: 0,
                col: 0,
                tok_type: TokenType::EOF,
            });
            Ok(0)
        }

        fn peek(&self, stack: Peek) -> &Token {
            match stack {
                Peek::Stack => &self.tokens[self.peek],
                Peek::Word(s, i) => {
                    let word = self.words.get(&s).unwrap();
                    return &word[i];
                }
            }
        }

        fn handle_while(
            &mut self,
            op: Box<TokenType>,
            pos: i32,
            tok: Token,
        ) -> Result<i32, String> {
            self.stack -= 1;
            let s = format!(
                "\t%c_{}_{} =d add 0, %s_main_{}\n",
                pos,
                pos,
                self.stack - 1
            );
            self.output_file.write(s.as_bytes()).unwrap();
            let s = format!(
                "\t%c_{}_{} =d add 0, %s_main_{}\n",
                pos,
                pos + 1,
                self.stack
            );
            self.output_file.write(s.as_bytes()).unwrap();
            let comp = match *op.to_owned() {
                TokenType::EQUAL => "eq",
                TokenType::NEQUAL => "ne",
                TokenType::LTE => "le",
                TokenType::LT => "lt",
                TokenType::GTE => "ge",
                TokenType::GT => "gt",
                _ => {
                    return self.format_err(
                        &tok,
                        format!("compiler: Error handling comparison: {:?}", tok),
                    )
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
            self.output_file.write(s.as_bytes()).unwrap();
            self.loop_stack += 1;
            self.loop_end_stack = self.loop_stack;

            let s = format!(
                "\tjnz %b, @loop_{}_{}, @end_loop_{}_{}\n@loop_{}_{}\n",
                pos, self.loop_end_stack, pos, self.loop_end_stack, pos, self.loop_end_stack
            );
            self.output_file.write(s.as_bytes()).unwrap();
            Ok(0)
        }

        fn handle_end(
            &mut self,
            cur_block: EndBlock,
            pos: i32,
            cond_str: String,
        ) -> Result<i32, String> {
            let s: String;
            match cur_block {
                EndBlock::Cond => {
                    if pos == 0 {
                        s = format!(
                            "@else_{}_{}\n@end_if_{}_{}\n",
                            pos, self.if_end_stack, pos, self.if_end_stack
                        );
                    } else {
                        s = format!("@end_if_{}_{}\n", pos, self.if_end_stack);
                    }
                    self.output_file.write(s.as_bytes()).unwrap();
                    self.if_end_stack -= 1;
                    if self.else_stack > 0 {
                        self.else_stack -= 1;
                    }
                }
                EndBlock::Loop => {
                    s = format!("\t%c_{}_{} =d sub %c_{}_{}, 1\n", pos, pos, pos, pos,);
                    self.output_file.write(s.as_bytes()).unwrap();
                    self.output_file.write(cond_str.as_bytes()).unwrap();
                    let s = format!(
                        "\tjnz %b, @loop_{}_{}, @end_loop_{}_{}\n@end_loop_{}_{}\n",
                        pos,
                        self.loop_end_stack,
                        pos,
                        self.loop_end_stack,
                        pos,
                        self.loop_end_stack
                    );
                    self.loop_end_stack -= 1;
                    self.output_file.write(s.as_bytes()).unwrap();
                }
            }
            Ok(0)
        }

        fn handle_word_call(&mut self, word: String) -> Result<i32, String> {
            let mut cond_str = "".to_string();
            let word_body = self.words.get(&word).unwrap().clone();
            for i in 0..word_body.len() {
                let res = self.handle_tokens(
                    word_body[i].clone(),
                    &mut cond_str,
                    Peek::Word(word.clone(), i + 1),
                );
                if let Err(e) = res {
                    return Err(e);
                }
            }
            Ok(0)
        }

        fn parse_function_body(&mut self, end: TokenType) -> Result<i32, String> {
            let mut cond_str = "".to_string();
            while self.tokens[self.pos].tok_type != end {
                let tok = self.tokens[self.pos].clone();
                let res = self.handle_tokens(tok, &mut cond_str, Peek::Stack);
                self.advance_token();
                if let Err(e) = res {
                    return Err(e);
                }
            }
            Ok(0)
        }

        fn handle_tokens(
            &mut self,
            tok: Token,
            cond_str: &mut String,
            peek_target: Peek,
        ) -> Result<i32, String> {
            match &tok.tok_type {
                TokenType::PLUS => match self.write_op("add") {
                    Ok(i) => self.stack = i,
                    Err(e) => return Err(e),
                },
                TokenType::MINUS => match self.write_op("sub") {
                    Ok(i) => self.stack = i,
                    Err(e) => return Err(e),
                },
                TokenType::ASTERISK => match self.write_op("mul") {
                    Ok(i) => self.stack = i,
                    Err(e) => return Err(e),
                },
                TokenType::SLASH => match self.write_op("div") {
                    Ok(i) => self.stack = i,
                    Err(e) => return Err(e),
                },
                TokenType::EQUAL => (self.stack, *cond_str) = self.comp_op("="),
                TokenType::NEQUAL => (self.stack, *cond_str) = self.comp_op("!="),
                TokenType::LTE => (self.stack, *cond_str) = self.comp_op("<="),
                TokenType::LT => (self.stack, *cond_str) = self.comp_op("<"),
                TokenType::GTE => (self.stack, *cond_str) = self.comp_op(">="),
                TokenType::GT => (self.stack, *cond_str) = self.comp_op(">"),
                TokenType::INT(_) => self.stack = self.push_op(&tok),
                TokenType::STR(s) => {
                    self.stack += 1;
                    self.strings.insert(s.to_string());
                    self.string_stack.insert(self.stack, s.to_string());
                }
                TokenType::DBG => self.dbg_op(),
                TokenType::SEMICOLON => self.stack = 0,
                TokenType::PRINT => match self.print_op(&tok) {
                    Ok(s) => self.stack = s,
                    Err(e) => return Err(e),
                },
                TokenType::CHAR => match self.print_op(&tok) {
                    Ok(s) => self.stack = s,
                    Err(e) => return Err(e),
                },
                TokenType::PERIOD => match self.print_op(&tok) {
                    Ok(s) => self.stack = s,
                    Err(e) => return Err(e),
                },
                TokenType::COMMA => match self.print_op(&tok) {
                    Ok(s) => self.stack = s + 1,
                    Err(e) => return Err(e),
                },
                TokenType::SET => {
                    let var = self.var_stack.last();
                    if let Some(v) = var {
                        self.stack = match self.set_op(v.clone(), tok) {
                            Ok(i) => i,
                            Err(e) => return Err(e),
                        };
                    } else {
                        return self.format_err(&tok, "No variable to assign to".to_string());
                    }
                }
                TokenType::IF(pos) => {
                    self.if_stack += 1;
                    self.if_end_stack = self.if_stack;
                    let s = format!(
                        "\t%b =w dtosi %s_main_{}\n\tjnz %b, @if_{}_{}, @else_{}_{}\n@if_{}_{}\n",
                        self.stack,
                        pos,
                        self.if_end_stack,
                        pos,
                        self.if_end_stack,
                        pos,
                        self.if_end_stack
                    );
                    self.output_file.write(s.as_bytes()).unwrap();
                }
                TokenType::ELSE(pos) => {
                    self.else_stack = self.if_end_stack;
                    let s = format!(
                        "\tjmp @end_if_{}_{}\n@else_{}_{}\n",
                        pos, self.else_stack, pos, self.else_stack
                    );
                    self.output_file.write(s.as_bytes()).unwrap();
                }
                TokenType::WHILE(op, pos) => match self.handle_while(op.to_owned(), *pos, tok) {
                    Ok(_) => (),
                    Err(e) => return Err(e),
                },
                TokenType::END(cur_block, pos) => {
                    match self.handle_end(cur_block.to_owned(), *pos, cond_str.to_owned()) {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    }
                }
                TokenType::COLON => {
                    let res = self.new_word_op();
                    if let Err(e) = res {
                        return Err(e);
                    }
                }
                TokenType::IDENT(ref s) => {
                    if self.words.contains_key(s) {
                        if self.peek(peek_target).tok_type == TokenType::SET {
                            return self.format_err(
                                &tok,
                                format!("Invalid assignment: {:?} is a word, not a variable", tok),
                            );
                        }

                        let res = self.handle_word_call(s.to_string());
                        if let Err(e) = res {
                            return Err(e);
                        }
                    } else {
                        match self.vars.insert(s.to_string()) {
                            true => {
                                if self.peek(peek_target).tok_type != TokenType::SET {
                                    return self
                                        .format_err(&tok, format!("Invalid: {:?} undefined", tok));
                                }
                                self.var_stack.push(format!("s_{}", s))
                            }
                            false => {
                                if self.peek(peek_target).tok_type != TokenType::SET {
                                    self.stack = self.push_op(&tok);
                                    self.var_stack.pop();
                                }

                                self.var_stack.push(format!("s_{}", s))
                            }
                        };
                    }
                }
                TokenType::LPAREN => (),
                TokenType::RPAREN => (),
                TokenType::EM => (),
                TokenType::EOF => (),
                _ => return self.format_err(&tok, format!("compiler: Unhandled token: {:?}", tok)),
            }

            Ok(0)
        }

        pub fn compile(&mut self) -> Result<i32, String> {
            self.output_file
                .write(b"export function w $main() {\n@start\n")
                .unwrap();

            self.cur_word = "main".to_string();
            let res = self.parse_function_body(TokenType::EOF);

            self.output_file.write(b"@end\n\tret 0\n}\n").unwrap();
            self.output_file
                .write(b"data $fmt_int = { b \"%.f \", b 0 }\n")
                .unwrap();
            self.output_file
                .write(b"data $fmt_dec = { b \"%.10g \", b 0 }\n")
                .unwrap();
            self.output_file
                .write(b"data $fmt_str = { b \"%s \", b 0 }\n")
                .unwrap();
            self.output_file
                .write(b"data $fmt_char = { b \"%c\", b 0 }\n")
                .unwrap();

            self.output_file
                .write(b"data $dbg = { b \"Debug: \", b 0 }\n")
                .unwrap();
            self.output_file
                .write(b"data $nl = { b \"\\n\", b 0 }\n")
                .unwrap();
            for k in &self.strings {
                let v = format!(
                    "data ${} = {{ b \"{}\", b 0 }}\n",
                    self.strings.get(k).unwrap().clone(),
                    self.strings.get(k).unwrap().clone()
                )
                .into_bytes();
                self.output_file.write(&v).unwrap();
            }

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
