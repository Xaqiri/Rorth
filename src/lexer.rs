pub mod lexer {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum TokenType {
        EOF,
        INT(u32),
        STR(String),
        PLUS,
        MINUS,
        ASTERISK,
        SLASH,
        PERIOD,
        COMMA,
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
        QMARK(usize),
        COLON(usize),
        WHILE(usize),
        END(usize),
    }

    #[derive(Debug)]
    pub struct Token {
        pub col: usize,
        pub row: usize,
        pub tok_type: TokenType,
    }

    pub struct Lexer {
        pos: usize,
        col: usize,
        row: usize,
        peek: usize,
        char: char,
        program: Vec<char>,
        ident: HashMap<String, TokenType>,
        pub tokens: Vec<Token>,
    }

    pub fn new(code: String) -> Lexer {
        let mut l = Lexer {
            pos: 0,
            col: 0,
            row: 0,
            peek: 1,
            char: ' ',
            program: code.chars().collect(),
            ident: HashMap::new(),
            tokens: vec![],
        };
        l.char = l.program[l.pos];
        l.ident.insert("swap".to_string(), TokenType::SWAP);
        l.ident.insert("over".to_string(), TokenType::OVER);
        l.ident.insert("peek".to_string(), TokenType::PEEK);
        l.ident.insert("drop".to_string(), TokenType::DROP);
        l.ident.insert("nip".to_string(), TokenType::NIP);
        l.ident.insert("dup".to_string(), TokenType::DUP);
        l.ident.insert("while".to_string(), TokenType::WHILE(0));
        l.ident.insert("end".to_string(), TokenType::END(0));
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
            if self.pos >= self.program.len() {
                self.char = '\0';
            } else {
                self.char = self.program[self.pos];
            }
        }

        pub fn skip_space(&mut self) {
            if self.char == '\0' {
                return;
            }
            if self.program[self.pos].is_whitespace() {
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
            while self.peek < self.program.len() && self.program[self.peek].is_alphabetic() {
                ident.push(self.program[self.peek]);
                self.advance_token();
            }
            let s = ident.iter().collect();
            if let Some(t) = self.ident.get(&s) {
                match t {
                    TokenType::SWAP => self.tokens.push(self.make_token(TokenType::SWAP)),
                    TokenType::OVER => self.tokens.push(self.make_token(TokenType::OVER)),
                    TokenType::PEEK => self.tokens.push(self.make_token(TokenType::PEEK)),
                    TokenType::DROP => self.tokens.push(self.make_token(TokenType::DROP)),
                    TokenType::DUP => self.tokens.push(self.make_token(TokenType::DUP)),
                    TokenType::NIP => self.tokens.push(self.make_token(TokenType::NIP)),
                    TokenType::WHILE(_) => self.tokens.push(self.make_token(TokenType::WHILE(0))),
                    TokenType::END(_) => self.tokens.push(self.make_token(TokenType::END(0))),
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
                    TokenType::QMARK(_) => println!("Invalid ident: {:?}", t),
                    TokenType::COLON(_) => println!("Invalid ident: {:?}", t),
                    TokenType::EOF => println!("Invalid ident: {:?}", t),
                }
            } else {
                println!("String not currently supported: {:?}", s);
                self.tokens.push(self.make_token(TokenType::STR(s)));
            }
        }

        pub fn get_number(&mut self) {
            let mut num = vec![];
            num.push(self.char);
            while self.peek < self.program.len() && self.program[self.peek].is_digit(10) {
                num.push(self.program[self.peek]);
                self.advance_token();
            }
            let s: String = num.iter().collect();
            let n = s.parse::<u32>();
            if let Ok(i) = n {
                self.tokens.push(self.make_token(TokenType::INT(i)))
            }
        }

        pub fn lex(&mut self) -> Result<(), String> {
            while self.char != '\0' {
                self.skip_space();
                match self.char {
                    '+' => self.tokens.push(self.make_token(TokenType::PLUS)),
                    '-' => self.tokens.push(self.make_token(TokenType::MINUS)),
                    '*' => self.tokens.push(self.make_token(TokenType::ASTERISK)),
                    '/' => self.tokens.push(self.make_token(TokenType::SLASH)),
                    '.' => self.tokens.push(self.make_token(TokenType::PERIOD)),
                    ',' => self.tokens.push(self.make_token(TokenType::COMMA)),
                    '=' => self.tokens.push(self.make_token(TokenType::EQUAL)),
                    '<' => self.tokens.push(self.make_token(TokenType::LT)),
                    '>' => self.tokens.push(self.make_token(TokenType::GT)),
                    '?' => self
                        .tokens
                        .push(self.make_token(TokenType::QMARK(self.pos))),
                    ':' => self
                        .tokens
                        .push(self.make_token(TokenType::COLON(self.pos))),
                    '\0' => self.tokens.push(self.make_token(TokenType::EOF)),
                    _ => {
                        if self.char.is_digit(10) {
                            self.get_number();
                        } else if self.char.is_alphabetic() {
                            self.get_ident();
                        } else {
                            return Err(format!(
                                "main.rs:{}:{}: Error lexing character: {}",
                                self.col, self.row, self.char
                            ));
                        }
                    }
                }
                self.advance_token();
            }
            Ok(())
        }
    }
}
