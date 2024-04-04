pub mod compiler {
    use crate::lexer::lexer::{Token, TokenType};

    #[derive(Debug)]
    #[repr(u8)]
    enum Op {
        NOOP = 0x00,
        PUSH = 0x01,
        DROP = 0x11,
        DUP = 0x21,
        PRINT = 0x31,
        SWAP = 0x02,
        NIP = 0x12,
        ROT = 0x03,
        ADD = 0x04,
        SUB = 0x14,
        MUL = 0x24,
        DIV = 0x34,
        HALT = 0xff,
    }

    pub struct Compiler {
        source_file: String,
        tokens: Vec<Token>,
        pub bytes: Vec<u8>,
        const_pool: Vec<i64>,
    }

    impl Compiler {
        pub fn compile(&mut self) -> Result<i32, String> {
            for i in &self.tokens {
                match i.tok_type {
                    TokenType::EOF => self.bytes.push(Op::HALT as u8),
                    TokenType::INT(n) => {
                        self.const_pool.push(n as i64);
                        self.bytes.push(Op::PUSH as u8);
                        self.bytes.push((self.const_pool.len() - 1) as u8);
                    }
                    TokenType::STR(_) => todo!(),
                    TokenType::IDENT(_) => todo!(),
                    TokenType::PLUS => self.bytes.push(Op::ADD as u8),
                    TokenType::MINUS => todo!(),
                    TokenType::ASTERISK => todo!(),
                    TokenType::SLASH => todo!(),
                    TokenType::PERIOD => self.bytes.push(Op::PRINT as u8),
                    TokenType::COMMA => todo!(),
                    TokenType::SET => todo!(),
                    TokenType::EQUAL => todo!(),
                    TokenType::NEQUAL => todo!(),
                    TokenType::LTE => todo!(),
                    TokenType::LT => todo!(),
                    TokenType::GTE => todo!(),
                    TokenType::GT => todo!(),
                    TokenType::PEEK => todo!(),
                    TokenType::DBG => todo!(),
                    TokenType::PRINT => todo!(),
                    TokenType::CHAR => todo!(),
                    TokenType::QMARK => todo!(),
                    TokenType::COLON => todo!(),
                    TokenType::SEMICOLON => todo!(),
                    TokenType::LPAREN => todo!(),
                    TokenType::RPAREN => todo!(),
                    TokenType::EM => todo!(),
                    TokenType::IF(_) => todo!(),
                    TokenType::ELSE(_) => todo!(),
                    TokenType::WHILE(_, _) => todo!(),
                    TokenType::END(_, _) => todo!(),
                }
            }
            for i in &self.bytes {
                println!("{:?}", i);
            }
            println!("");
            Ok(0)
        }
    }

    pub fn new(source_file: String, tokens: Vec<Token>) -> Compiler {
        Compiler {
            source_file,
            tokens,
            bytes: vec![],
            const_pool: vec![],
        }
    }
}
