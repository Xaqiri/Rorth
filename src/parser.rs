pub mod parser {
    use std::collections::HashSet;

    use crate::lexer::lexer::{EndBlock, Token, TokenType};

    pub struct Parser {
        pos: usize,
        peek: usize,
        source_file: String,
        stack: i32,
        var_stack: Vec<String>,
        vars: HashSet<String>,
        if_stack: i32,
        else_stack: i32,
        loop_stack: i32,
        if_end_stack: i32,
        loop_end_stack: i32,
        cur_block: EndBlock,
        parse_inside: bool,
        pub tokens: Vec<Token>,
    }

    pub fn new(source: String, tokens: Vec<Token>) -> Parser {
        let p = Parser {
            pos: 0,
            peek: 1,
            source_file: source,
            stack: 0,
            var_stack: vec![],
            vars: HashSet::new(),
            if_stack: 0,
            else_stack: 0,
            loop_stack: 0,
            if_end_stack: 0,
            loop_end_stack: 0,
            cur_block: EndBlock::Cond,
            parse_inside: true,
            tokens,
        };
        p
    }

    impl Parser {
        pub fn print(&self) {
            for i in &self.tokens {
                println!("{:?}", i);
            }
        }

        fn advance_token(&mut self) {
            self.pos += 1;
            self.peek += 1;
        }

        // fn peek(&self) -> &Token {
        //     &self.tokens[self.peek]
        // }

        fn stack_overflow(&mut self, tok: &Token, req: i32, change: i32) -> Result<i32, String> {
            if self.parse_inside {
                if self.stack < req {
                    return Err(format!(
                        "{}:{}:{}: Invalid {:?}: Not enough values on the stack",
                        self.source_file, tok.col, tok.row, tok.tok_type
                    ));
                }
                return Ok(self.stack + change);
            } else {
                return Ok(self.stack);
            }
        }

        fn parse_ident(&mut self, var: &String) -> Result<i32, String> {
            match self.vars.insert(var.to_string()) {
                true => {
                    self.var_stack.push(var.to_string());
                    Ok(self.stack)
                }
                false => Ok(self.stack + 1),
            }
        }

        fn parse_set(&mut self) -> Result<i32, String> {
            let var = self.var_stack.last();
            if let Some(_) = var {
                return Ok(self.stack - 1);
            } else {
                return Err("No variable to assign to".to_string());
            }
        }

        fn parse_if_block(&mut self, tok: &Token, req: i32, change: i32) -> Result<Token, String> {
            let err = self.stack_overflow(tok, req, change);
            match err {
                Ok(i) => {
                    if i < 0 {
                        panic!(
                            "{}",
                            format!(
                                "{}:{}:{}: Invalid {:?}: Nothing on the stack to compare",
                                self.source_file, tok.row, tok.col, tok.tok_type
                            )
                        );
                    }
                    let new_tok: Token;
                    if let TokenType::IF(_) = tok.tok_type {
                        self.if_stack += 1;
                        self.if_end_stack = self.if_stack;
                        new_tok = Token {
                            col: tok.col,
                            row: tok.row,
                            tok_type: TokenType::IF(self.if_end_stack),
                        };
                    } else {
                        self.else_stack = self.if_end_stack;
                        if self.else_stack == 0 {
                            return Err(format!(
                                "{}:{}:{}: Invalid {:?}: Can't use {:?} without preceding IF",
                                self.source_file, tok.col, tok.row, tok.tok_type, tok.tok_type
                            ));
                        }

                        new_tok = Token {
                            col: tok.col,
                            row: tok.row,
                            tok_type: TokenType::ELSE(self.else_stack),
                        };
                    }
                    self.cur_block = EndBlock::Cond;

                    Ok(new_tok)
                }
                Err(e) => Err(e),
            }
        }

        fn parse_while_block(
            &mut self,
            tok: &Token,
            req: i32,
            change: i32,
        ) -> Result<Token, String> {
            let err = self.stack_overflow(tok, req, change);
            match err {
                Ok(i) => {
                    if i < 1 {
                        panic!(
                            "{}",
                            format!(
                                "Invalid {:?}: Nothing on the stack to compare",
                                tok.tok_type
                            )
                        );
                    }
                    let op = &self.tokens[self.pos - 1].tok_type;
                    let new_tok: Token;
                    self.loop_stack += 1;
                    self.loop_end_stack = self.loop_stack;
                    new_tok = Token {
                        col: tok.col,
                        row: tok.row,
                        tok_type: TokenType::WHILE(Box::new(op.clone()), self.loop_end_stack),
                    };
                    self.cur_block = EndBlock::Loop;
                    return Ok(new_tok);
                }
                Err(e) => Err(e),
            }
        }

        fn parse_end(&mut self, tok: &Token) -> Result<Token, String> {
            let new_tok: Token;
            match self.cur_block {
                EndBlock::Cond => {
                    new_tok = Token {
                        col: tok.col,
                        row: tok.row,
                        tok_type: TokenType::END(EndBlock::Cond, self.if_end_stack),
                    };

                    if self.if_end_stack == 0 {
                        return Err(format!(
                            "{}:{}:{}: Error parsing {:?}: {:?} without matching IF",
                            self.source_file, tok.row, tok.col, tok.tok_type, tok.tok_type
                        ));
                    }
                    self.if_end_stack -= 1;
                    if self.else_stack > 0 {
                        self.else_stack -= 1;
                    }
                    self.stack -= 2;
                }
                EndBlock::Loop => {
                    new_tok = Token {
                        col: tok.col,
                        row: tok.row,
                        tok_type: TokenType::END(EndBlock::Loop, self.loop_end_stack),
                    };
                    if self.loop_end_stack == 0 {
                        return Err(format!(
                            "{}:{}:{}: Error parsing {:?}: {:?} without matching WHILE",
                            self.source_file, tok.row, tok.col, tok.tok_type, tok.tok_type
                        ));
                    }
                    self.loop_end_stack -= 1;
                    self.stack -= 2;
                }
            }
            Ok(new_tok)
        }

        fn parse_matched_token(&mut self, tok: &Token, tok_type: TokenType) -> Result<u32, String> {
            let pos = self.pos;
            let peek = self.peek;
            while self.tokens[self.pos].tok_type != tok_type {
                self.advance_token();
                if self.tokens[self.pos].tok_type == TokenType::EOF {
                    return Err(format!(
                        "{}:{}:{}: Error parsing {:?}: {:?} without closing {:?}",
                        self.source_file, tok.row, tok.col, tok.tok_type, tok.tok_type, tok_type
                    ));
                }
            }
            self.pos = pos;
            self.peek = peek;
            Ok(0)
        }

        fn parse_word(&mut self, tok: &Token) -> Result<u32, String> {
            self.parse_matched_token(tok, TokenType::SEMICOLON)
        }

        pub fn parse(&mut self) -> Result<u32, String> {
            while self.tokens[self.pos].tok_type != TokenType::EOF {
                let tok = &self.tokens[self.pos].clone();
                let err = match &tok.tok_type {
                    TokenType::PLUS => self.stack_overflow(tok, 2, -1),
                    TokenType::MINUS => self.stack_overflow(tok, 2, -1),
                    TokenType::ASTERISK => self.stack_overflow(tok, 2, -1),
                    TokenType::SLASH => self.stack_overflow(tok, 2, -1),
                    TokenType::EQUAL => self.stack_overflow(tok, 2, 1),
                    TokenType::NEQUAL => self.stack_overflow(tok, 2, 1),
                    TokenType::LTE => self.stack_overflow(tok, 2, 1),
                    TokenType::LT => self.stack_overflow(tok, 2, 1),
                    TokenType::GTE => self.stack_overflow(tok, 2, 1),
                    TokenType::GT => self.stack_overflow(tok, 2, 1),
                    TokenType::SWAP => self.stack_overflow(tok, 2, 0),
                    TokenType::DROP => self.stack_overflow(tok, 1, -1),
                    TokenType::NIP => self.stack_overflow(tok, 2, -1),
                    TokenType::ROT => self.stack_overflow(tok, 3, 0),
                    TokenType::OVER => self.stack_overflow(tok, 2, 1),
                    TokenType::DUP => self.stack_overflow(tok, 1, 1),
                    TokenType::PERIOD => self.stack_overflow(tok, 1, -1),
                    TokenType::COMMA => self.stack_overflow(tok, 1, 0),
                    TokenType::SET => self.parse_set(),
                    TokenType::DBG => Ok(self.stack),
                    TokenType::IF(_) => match self.parse_if_block(tok, 1, -1) {
                        Ok(t) => {
                            self.tokens[self.pos] = t;
                            Ok(self.stack)
                        }
                        Err(e) => Err(e),
                    },
                    TokenType::ELSE(_) => match self.parse_if_block(tok, 1, 0) {
                        Ok(t) => {
                            self.tokens[self.pos] = t;
                            Ok(self.stack)
                        }
                        Err(e) => Err(e),
                    },
                    TokenType::END(_, _) => match self.parse_end(tok) {
                        Ok(t) => {
                            self.tokens[self.pos] = t;
                            Ok(self.stack)
                        }
                        Err(e) => Err(e),
                    },
                    TokenType::INT(_) => Ok(self.stack + 1),
                    TokenType::WHILE(_, _) => match self.parse_while_block(tok, 1, 0) {
                        Ok(t) => {
                            self.tokens[self.pos] = t;
                            Ok(self.stack)
                        }
                        Err(e) => Err(e),
                    },
                    TokenType::IDENT(s) => match self.parse_ident(&s) {
                        Ok(i) => Ok(i),
                        Err(e) => Err(e),
                    },
                    TokenType::COLON => {
                        self.parse_inside = false;
                        match self.parse_word(tok) {
                            Ok(_) => Ok(0),
                            Err(e) => Err(e),
                        }
                    }
                    TokenType::SEMICOLON => {
                        self.parse_inside = true;
                        Ok(0)
                    }
                    TokenType::LPAREN => match self.parse_matched_token(tok, TokenType::RPAREN) {
                        Ok(_) => Ok(0),
                        Err(e) => Err(e),
                    },
                    TokenType::RPAREN => Ok(0),
                    TokenType::EM => Ok(0),

                    TokenType::EOF => {
                        if self.if_end_stack > 0 {
                            Err(format!(
                                "{}:{}:{}: Unclosed IF",
                                self.source_file, tok.row, tok.col
                            ))
                        } else {
                            break;
                        }
                    }
                    _ => return Err(format!("parser: Unhandled token: {:?}", tok)),
                };
                match err {
                    Ok(i) => {
                        self.stack = i;
                        self.advance_token();
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(0)
        }
    }
}
