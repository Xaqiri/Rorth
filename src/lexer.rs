pub mod lexer {
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum EndBlock {
        Cond,
        Loop,
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub enum TokenType {
        EOF,
        INT(u32),
        STR(String),
        IDENT(String),
        PLUS,
        MINUS,
        ASTERISK,
        SLASH,
        PERIOD,
        COMMA,
        SET,
        EQUAL,
        NEQUAL,
        LTE,
        LT,
        GTE,
        GT,
        SWAP,
        OVER,
        PEEK,
        DROP,
        DUP,
        NIP,
        ROT,
        DBG,
        PRINT,
        QMARK,
        COLON,
        SEMICOLON,
        LPAREN,
        RPAREN,
        EM,
        IF(i32),
        ELSE(i32),
        WHILE(Box<TokenType>, i32),
        END(EndBlock, i32),
    }

    #[derive(Debug, Clone)]
    pub struct Token {
        pub col: usize,
        pub row: usize,
        pub tok_type: TokenType,
    }
    impl Token {
        pub fn new() -> Token {
            Token {
                col: 0,
                row: 0,
                tok_type: TokenType::EOF,
            }
        }
    }

    pub struct Lexer {
        pos: usize,
        col: usize,
        row: usize,
        peek: usize,
        char: char,
        source: Vec<char>,
        source_file: String,
        ident: HashMap<String, TokenType>,
        pub tokens: Vec<Token>,
    }

    pub fn new(source_file: String, code: String) -> Lexer {
        let mut l = Lexer {
            pos: 0,
            col: 1,
            row: 1,
            peek: 1,
            char: ' ',
            source: code.chars().collect(),
            source_file,
            ident: HashMap::new(),
            tokens: vec![],
        };
        l.char = l.source[l.pos];
        l.ident.insert("swap".to_string(), TokenType::SWAP);
        l.ident.insert("over".to_string(), TokenType::OVER);
        l.ident.insert("peek".to_string(), TokenType::PEEK);
        l.ident.insert("drop".to_string(), TokenType::DROP);
        l.ident.insert("nip".to_string(), TokenType::NIP);
        l.ident.insert("rot".to_string(), TokenType::ROT);
        l.ident.insert("dup".to_string(), TokenType::DUP);
        l.ident.insert("dbg".to_string(), TokenType::DBG);
        l.ident.insert("print".to_string(), TokenType::PRINT);
        l.ident.insert("set".to_string(), TokenType::SET);
        l.ident.insert("if".to_string(), TokenType::IF(0));
        l.ident.insert("else".to_string(), TokenType::ELSE(0));
        l.ident.insert(
            "while".to_string(),
            TokenType::WHILE(Box::new(TokenType::EQUAL), 0),
        );
        l.ident
            .insert("end".to_string(), TokenType::END(EndBlock::Cond, 0));
        l
    }

    impl Lexer {
        pub fn print(&self) {
            for i in &self.tokens {
                println!("{:?}", i);
            }
        }

        pub fn advance_token(&mut self) {
            self.pos += 1;
            self.col += 1;
            self.peek += 1;
            if self.char == '\n' {
                self.row += 1;
                self.col = 1;
            }
            if self.pos >= self.source.len() {
                self.char = '\0';
                return;
            } else {
                self.char = self.source[self.pos];
            }
        }

        pub fn skip_space(&mut self) {
            if self.char == '\0' {
                return;
            }
            if self.source[self.pos].is_whitespace() {
                self.advance_token();
                self.skip_space();
            }
        }

        pub fn make_token(&self, token_type: TokenType) -> Token {
            Token {
                col: self.col,
                row: self.row,
                tok_type: token_type,
            }
        }

        pub fn get_ident(&mut self) {
            let mut ident = vec![];
            ident.push(self.char);
            while self.peek < self.source.len() && self.source[self.peek].is_alphabetic() {
                ident.push(self.source[self.peek]);
                self.advance_token();
            }
            let s = ident.into_iter().collect();
            if let Some(t) = self.ident.get(&s) {
                match t {
                    TokenType::SWAP => self.tokens.push(self.make_token(TokenType::SWAP)),
                    TokenType::OVER => self.tokens.push(self.make_token(TokenType::OVER)),
                    TokenType::PEEK => self.tokens.push(self.make_token(TokenType::PEEK)),
                    TokenType::DROP => self.tokens.push(self.make_token(TokenType::DROP)),
                    TokenType::DUP => self.tokens.push(self.make_token(TokenType::DUP)),
                    TokenType::NIP => self.tokens.push(self.make_token(TokenType::NIP)),
                    TokenType::ROT => self.tokens.push(self.make_token(TokenType::ROT)),
                    TokenType::DBG => self.tokens.push(self.make_token(TokenType::DBG)),
                    TokenType::PRINT => self.tokens.push(self.make_token(TokenType::PRINT)),
                    TokenType::IF(_) => self.tokens.push(self.make_token(TokenType::IF(0))),
                    TokenType::ELSE(_) => self.tokens.push(self.make_token(TokenType::ELSE(0))),
                    TokenType::WHILE(_, _) => self
                        .tokens
                        .push(self.make_token(TokenType::WHILE(Box::new(TokenType::EQUAL), 0))),
                    TokenType::END(_, _) => self
                        .tokens
                        .push(self.make_token(TokenType::END(EndBlock::Cond, 0))),
                    TokenType::SET => self.tokens.push(self.make_token(TokenType::SET)),
                    TokenType::IDENT(s) => println!("Invalid ident: {:?} ({})", t, s),
                    TokenType::INT(_) => println!("Invalid ident: {:?}", t),
                    TokenType::STR(_) => println!("Invalid ident: {:?}", t),
                    TokenType::PLUS => println!("Invalid ident: {:?}", t),
                    TokenType::MINUS => println!("Invalid ident: {:?}", t),
                    TokenType::ASTERISK => println!("Invalid ident: {:?}", t),
                    TokenType::SLASH => println!("Invalid ident: {:?}", t),
                    TokenType::PERIOD => println!("Invalid ident: {:?}", t),
                    TokenType::COMMA => println!("Invalid ident: {:?}", t),
                    TokenType::EQUAL => println!("Invalid ident: {:?}", t),
                    TokenType::NEQUAL => println!("Invalid ident: {:?}", t),
                    TokenType::LTE => println!("Invalid ident: {:?}", t),
                    TokenType::LT => println!("Invalid ident: {:?}", t),
                    TokenType::GTE => println!("Invalid ident: {:?}", t),
                    TokenType::GT => println!("Invalid ident: {:?}", t),
                    TokenType::QMARK => println!("Invalid ident: {:?}", t),
                    TokenType::COLON => println!("Invalid ident: {:?}", t),
                    TokenType::SEMICOLON => println!("Invalid ident: {:?}", t),
                    TokenType::EM => println!("Invalid ident: {:?}", t),
                    TokenType::LPAREN => println!("Invalid ident: {:?}", t),
                    TokenType::RPAREN => println!("Invalid ident: {:?}", t),
                    TokenType::EOF => println!("Invalid ident: {:?}", t),
                }
            } else {
                self.tokens.push(self.make_token(TokenType::IDENT(s)));
            }
        }

        pub fn get_number(&mut self) {
            let mut num = vec![];
            num.push(self.char);
            while self.peek < self.source.len() && self.source[self.peek].is_digit(10) {
                num.push(self.source[self.peek]);
                self.advance_token();
            }
            let s: String = num.iter().collect();
            let n = s.parse::<u32>();
            if let Ok(i) = n {
                self.tokens.push(self.make_token(TokenType::INT(i)))
            }
        }

        fn peek(&self) -> char {
            self.source[self.peek]
        }

        fn parse_comment(&mut self) {
            let row = self.row;
            let col = self.col;
            if self.char == '(' {
                while self.char != ')' {
                    if self.char == '\0' {
                        panic!(
                            "{}",
                            format!("{}:{}:{}: ( without closing )", self.source_file, row, col)
                        );
                    }

                    self.advance_token();
                }
            } else if self.char == '\\' {
                while self.char != '\n' {
                    self.advance_token();
                }
            }
        }

        pub fn lex(&mut self) -> Result<Vec<Token>, String> {
            while self.pos < self.source.len() {
                self.skip_space();
                match self.char {
                    '+' => self.tokens.push(self.make_token(TokenType::PLUS)),
                    '*' => self.tokens.push(self.make_token(TokenType::ASTERISK)),
                    '/' => self.tokens.push(self.make_token(TokenType::SLASH)),
                    '.' => self.tokens.push(self.make_token(TokenType::PERIOD)),
                    ',' => self.tokens.push(self.make_token(TokenType::COMMA)),
                    '=' => self.tokens.push(self.make_token(TokenType::EQUAL)),
                    '<' => self.tokens.push(self.make_token(TokenType::LT)),
                    '>' => self.tokens.push(self.make_token(TokenType::GT)),
                    '?' => self.tokens.push(self.make_token(TokenType::QMARK)),
                    ';' => self.tokens.push(self.make_token(TokenType::SEMICOLON)),
                    '(' | '\\' => self.parse_comment(),
                    ')' => self.tokens.push(self.make_token(TokenType::RPAREN)),
                    '-' => {
                        if self.peek() == '-' {
                            self.tokens.push(self.make_token(TokenType::EM));
                            self.advance_token();
                        } else {
                            self.tokens.push(self.make_token(TokenType::MINUS))
                        }
                    }
                    ':' => {
                        if self.peek() == '=' {
                            self.tokens.push(self.make_token(TokenType::SET));
                            self.advance_token();
                        } else {
                            self.tokens.push(self.make_token(TokenType::COLON));
                        }
                    }
                    '\0' => self.tokens.push(self.make_token(TokenType::EOF)),
                    _ => {
                        if self.char.is_digit(10) {
                            self.get_number();
                        } else if self.char.is_alphabetic() {
                            self.get_ident();
                        } else {
                            return Err(format!(
                                "{}:{}:{}: Error lexing character: {}",
                                self.source_file, self.row, self.col, self.char
                            ));
                        }
                    }
                }
                self.advance_token();
            }
            Ok(self.tokens.to_vec())
        }
    }
}
