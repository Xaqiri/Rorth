pub mod lexer {

    #[derive(Debug, Clone, Copy)]
    pub enum TokenType {
        EOF,
        INT(u32),
        PLUS,
        MINUS,
        ASTERISK,
        SLASH,
        PERIOD,
        COMMA,
        EQUAL,
    }

    #[derive(Debug, Clone, Copy)]
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
        pub tokens: Vec<Token>,
        program: Vec<char>,
    }

    pub fn new(code: String) -> Lexer {
        let mut l = Lexer {
            pos: 0,
            col: 0,
            row: 0,
            peek: 1,
            program: code.chars().collect(),
            char: ' ',
            tokens: vec![],
        };
        l.char = l.program[l.pos];
        l
    }

    impl Lexer {
        pub fn print(self) {
            for i in self.tokens.clone() {
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

        pub fn get_number(&mut self, i: char) {
            let mut num = vec![];
            num.push(i);
            while self.program[self.peek].is_digit(10) {
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
                    '\0' => self.tokens.push(self.make_token(TokenType::EOF)),
                    _ => {
                        let n = self.char.to_digit(10);
                        match n {
                            Some(_) => self.get_number(self.char),
                            None => {
                                return Err(format!(
                                    "main.rs: {}, {}: Error lexing character: {}",
                                    self.col, self.row, self.char
                                ))
                            }
                        }
                    }
                }
                self.advance_token();
            }
            Ok(())
        }
    }
}
