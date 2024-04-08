pub mod op {
    #[derive(Debug, Clone, Copy)]
    #[repr(u8)]
    pub enum Op {
        NOOP = 0x00,
        PUSHNUM(usize) = 0x01,
        PUSHSTR(usize) = 0x11,
        DROP = 0x21,
        DUP = 0x31,
        SWAP = 0x02,
        NIP = 0x12,
        ROT = 0x03,
        ADD = 0x04,
        SUB = 0x14,
        MUL = 0x24,
        DIV = 0x34,
        DBG = 0xde,
        CONST(i64) = 0xee,
        PRINTI = 0x0f,
        PRINTS = 0x2f,
        HALT = 0xff,
    }

    impl std::fmt::Display for Op {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Op::NOOP => write!(f, "00"),
                Op::PUSHNUM(n) => write!(f, "01 {:02x}", n),
                Op::PUSHSTR(n) => write!(f, "11 {:02x}", n),
                Op::SWAP => write!(f, "02"),
                Op::ROT => write!(f, "03"),
                Op::ADD => write!(f, "04"),
                Op::SUB => write!(f, "14"),
                Op::MUL => write!(f, "24"),
                Op::DIV => write!(f, "34"),
                Op::PRINTI => write!(f, "0f"),
                Op::PRINTS => write!(f, "2f"),
                Op::NIP => write!(f, "12"),
                Op::DROP => write!(f, "21"),
                Op::DUP => write!(f, "31"),
                Op::DBG => write!(f, "de"),
                Op::HALT => write!(f, "ff"),
                Op::CONST(_) => todo!(),
            }
        }
    }
}
