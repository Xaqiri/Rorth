pub mod vm {
    pub struct VM {
        bytes: Vec<u8>,
        ip: usize,
    }

    impl VM {
        pub fn disassemble(&mut self) -> Result<i32, String> {
            self.ip = 0;
            while self.ip < self.bytes.len() {
                match self.bytes[self.ip] {
                    0x00 => {}
                    0x01 => {
                        println!("PUSH");
                        self.ip += 1;
                    }
                    // 0x11 => {}
                    // 0x21 => {}
                    0x31 => println!("PRINT"),
                    // 0x02 => {}
                    // 0x12 => {}
                    // 0x03 => {}
                    0x04 => println!("ADD"),
                    // 0x14 => {}
                    // 0x24 => {}
                    // 0x34 => {}
                    0xff => println!("HALT"),
                    _ => {
                        return Err(format!(
                            "Unknown instruction: 0x{:02x}",
                            self.bytes[self.ip]
                        ))
                    }
                }
                self.ip += 1;
            }
            Ok(1)
        }
    }

    pub fn new(bytes: Vec<u8>) -> VM {
        VM { bytes, ip: 0 }
    }
}
