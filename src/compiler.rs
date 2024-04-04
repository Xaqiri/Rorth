pub mod compiler {
    use crate::{
        lexer::lexer::{Token, TokenType},
        op::op::Op,
    };

    pub struct Compiler {
        source_file: String,
        tokens: Vec<Token>,
        pub bytes: Vec<Op>,
        pub const_pool: Vec<i64>,
    }

    impl Compiler {
        pub fn compile(&mut self) -> Result<i32, String> {
            for i in &self.tokens {
                match i.tok_type {
                    TokenType::EOF => self.bytes.push(Op::HALT),
                    TokenType::INT(n) => {
                        self.const_pool.push(n as i64);
                        self.bytes.push(Op::PUSH(self.const_pool.len() - 1));
                    }
                    TokenType::STR(_) => todo!("{:?}", i),
                    TokenType::IDENT(_) => todo!("{:?}", i),
                    TokenType::PLUS => self.bytes.push(Op::ADD),
                    TokenType::MINUS => todo!("{:?}", i),
                    TokenType::ASTERISK => self.bytes.push(Op::MUL),
                    TokenType::SLASH => self.bytes.push(Op::DIV),
                    TokenType::PERIOD => self.bytes.push(Op::PRINT),
                    TokenType::COMMA => todo!("{:?}", i),
                    TokenType::SET => todo!("{:?}", i),
                    TokenType::EQUAL => todo!("{:?}", i),
                    TokenType::NEQUAL => todo!("{:?}", i),
                    TokenType::LTE => todo!("{:?}", i),
                    TokenType::LT => todo!("{:?}", i),
                    TokenType::GTE => todo!("{:?}", i),
                    TokenType::GT => todo!("{:?}", i),
                    TokenType::DUP => self.bytes.push(Op::DUP),
                    TokenType::SWAP => self.bytes.push(Op::SWAP),
                    TokenType::NIP => self.bytes.push(Op::NIP),
                    TokenType::ROT => self.bytes.push(Op::ROT),
                    TokenType::PEEK => todo!("{:?}", i),
                    TokenType::DBG => self.bytes.push(Op::DBG),
                    TokenType::PRINT => todo!("{:?}", i),
                    TokenType::CHAR => todo!("{:?}", i),
                    TokenType::QMARK => todo!("{:?}", i),
                    TokenType::COLON => todo!("{:?}", i),
                    TokenType::SEMICOLON => todo!("{:?}", i),
                    TokenType::LPAREN => todo!("{:?}", i),
                    TokenType::RPAREN => todo!("{:?}", i),
                    TokenType::EM => todo!("{:?}", i),
                    TokenType::IF(_) => todo!("{:?}", i),
                    TokenType::ELSE(_) => todo!("{:?}", i),
                    TokenType::WHILE(_, _) => todo!("{:?}", i),
                    TokenType::END(_, _) => todo!("{:?}", i),
                }
            }
            println!("COMPILER:");
            for i in &self.bytes {
                print!("{} ", i);
            }
            println!("\n");
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
