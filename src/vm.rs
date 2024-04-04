pub mod vm {
    use crate::op::op::Op;

    pub struct VM {
        bytes: Vec<Op>,
        ip: usize,
        sp: usize,
        stack: Vec<i64>,
        const_pool: Vec<i64>,
    }

    impl VM {
        fn math_op(&mut self, op: &Op) -> Result<(), String> {
            let b = self.stack.pop();
            self.sp -= 1;
            match b {
                Some(b) => match op {
                    Op::ADD => self.stack[self.sp] = self.stack[self.sp] + b,
                    Op::SUB => self.stack[self.sp] = self.stack[self.sp] - b,
                    Op::MUL => self.stack[self.sp] = self.stack[self.sp] * b,
                    Op::DIV => self.stack[self.sp] = self.stack[self.sp] / b,
                    _ => {}
                },
                None => return Err(format!("Invalid {:?}: Not enough values on the stack", op)),
            }
            Ok(())
        }

        fn push_op(&mut self, cidx: Option<usize>) -> Result<(), String> {
            if let Some(n) = cidx {
                self.stack.push(self.const_pool[n]);
            } else {
                self.stack.push(self.stack[self.sp]);
            }
            self.sp = self.stack.len() - 1;
            Ok(())
        }

        fn swap_op(&mut self) -> Result<(), String> {
            let tmp = self.stack[self.sp - 1];
            self.stack[self.sp - 1] = self.stack[self.sp];
            self.stack[self.sp] = tmp;

            Ok(())
        }

        fn rot_op(&mut self) -> Result<(), String> {
            let tmp = self.stack[self.sp - 2];
            self.stack[self.sp - 2] = self.stack[self.sp - 1];
            self.stack[self.sp - 1] = self.stack[self.sp];
            self.stack[self.sp] = tmp;

            Ok(())
        }

        pub fn interpret(&mut self) -> Result<(), String> {
            self.ip = 0;
            while self.ip < self.bytes.len() {
                match self.bytes[self.ip] {
                    Op::NOOP => Ok(()),
                    Op::PUSH(n) => self.push_op(Some(n)),
                    Op::DUP => self.push_op(None),
                    Op::SWAP => self.swap_op(),
                    Op::ROT => self.rot_op(),
                    Op::ADD => self.math_op(&self.bytes[self.ip].clone()),
                    Op::SUB => self.math_op(&self.bytes[self.ip].clone()),
                    Op::MUL => self.math_op(&self.bytes[self.ip].clone()),
                    Op::DIV => self.math_op(&self.bytes[self.ip].clone()),
                    Op::DBG => {
                        for i in &self.stack {
                            print!("{} ", i);
                        }
                        println!("");
                        Ok(())
                    }
                    Op::HALT => Ok(()),
                    _ => return Err(format!("Unknown instruction: {:?}", self.bytes[self.ip])),
                }?;
                self.ip += 1;
            }
            Ok(())
        }

        pub fn disassemble(&mut self) -> Result<i32, String> {
            println!("DISSASSEMBLE:");
            println!("{:?}", self.bytes);
            self.ip = 0;
            while self.ip < self.bytes.len() {
                match self.bytes[self.ip] {
                    Op::NOOP => {}
                    Op::PUSH(n) => println!("PUSH {}", self.const_pool[n]),
                    // 0x11 => {}
                    // 0x21 => {}
                    Op::PRINT => println!("PRINT"),
                    // 0x02 => {}
                    // 0x12 => {}
                    // 0x03 => {}
                    Op::ADD => println!("ADD"),
                    Op::SUB => println!("SUB"),
                    Op::MUL => println!("MUL"),
                    Op::DIV => println!("DIV"),
                    Op::DBG => println!("DBG"),
                    Op::HALT => println!("HALT"),
                    _ => return Err(format!("Unknown instruction: {:?}", self.bytes[self.ip])),
                }
                self.ip += 1;
            }
            Ok(0)
        }
    }

    pub fn new(bytes: Vec<Op>, const_pool: Vec<i64>) -> VM {
        VM {
            bytes,
            ip: 0,
            sp: 0,
            stack: vec![],
            const_pool,
        }
    }
}
