use crate::ops::*;

pub trait OpCodeDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode;
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    MemoryNoDisplacement,
    MemoryEightBitDisplacement,
    MemorySixteenBitDisplacement,
    Register,
}

#[derive(Clone, Copy)]
pub struct MovDecoder {}

impl MovDecoder {
    const DIR_MASK: u8 = 0b0000_0010;
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for MovDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let dir = decode_dir(code, Self::DIR_MASK);
        let width = decode_width(code, Self::WIDTH_MASK);

        let next = bytes.next().unwrap();
        let mode = decode_mode((*next >> 6) & 0b0000_0011);
        let reg = decode_reg((*next >> 3) & 0b0000_0111, width);

        let reg_or_mem = decode_reg_or_mem(next & 0b0000_0111, mode, width, bytes);

        OpCode::MovToFromRegMem {
            dir,
            reg,
            reg_or_mem,
        }
    }
}

#[derive(Clone)]
pub struct ImmediateMovToRegMemDecoder {}

impl ImmediateMovToRegMemDecoder {
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ImmediateMovToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let width = decode_width(code, Self::WIDTH_MASK);

        let next = bytes.next().unwrap();

        let mode = decode_mode((*next >> 6) & 0b0000_0011);
        let reg_or_mem = decode_reg_or_mem(next & 0b0000_0111, mode, width, bytes);
        let data = decode_immediate(bytes, width);

        OpCode::ImmediateMovRegMem {
            width,
            reg_or_mem,
            data,
        }
    }
}

#[derive(Clone)]
pub struct ImmediateMovToRegDecoder {}

impl ImmediateMovToRegDecoder {
    const WIDTH_MASK: u8 = 0b0000_1000;
}

impl OpCodeDecoder for ImmediateMovToRegDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let width = decode_width(code, Self::WIDTH_MASK);
        let reg = decode_reg(code & 0b0000_0111, width);
        let data = decode_immediate(bytes, width);
        OpCode::ImmediateMovReg { reg, data }
    }
}

#[derive(Clone)]
pub struct MovAccumulatorDecoder {}

impl MovAccumulatorDecoder {
    const WIDTH_MASK: u8 = 0b0000_0001;
    const DIR_MASK: u8 = 0b0000_0010;
}

impl OpCodeDecoder for MovAccumulatorDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let width = decode_width(code, Self::WIDTH_MASK);
        let dir = decode_dir(code, Self::DIR_MASK);
        let address = decode_address(bytes, width);
        OpCode::AccumulatorMove { dir, addr: address }
    }
}

#[derive(Clone)]
pub struct JumpDecoder {
    jump_op: fn(i8) -> OpCode,
}

impl JumpDecoder {
    pub fn new(op: fn(i8) -> OpCode) -> JumpDecoder {
        JumpDecoder { jump_op: op }
    }
}

impl OpCodeDecoder for JumpDecoder {
    fn decode(&self, _code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let disp = i8::from_le_bytes([*bytes.next().unwrap()]);
        (self.jump_op)(disp)
    }
}

#[derive(Clone)]
pub struct ArithmeticFromToRegMemDecoder {}

impl ArithmeticFromToRegMemDecoder {
    const DIR_MASK: u8 = 0b0000_0010;
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ArithmeticFromToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let dir = decode_dir(code, Self::DIR_MASK);
        let width = decode_width(code, Self::WIDTH_MASK);

        let op = decode_arithmetic_op((code >> 3) & 0b0000_0111);

        let next = bytes.next().unwrap();
        let mode = decode_mode((*next >> 6) & 0b0000_0011);
        let reg = decode_reg((*next >> 3) & 0b0000_0111, width);
        let reg_or_mem = decode_reg_or_mem(next & 0b0000_0111, mode, width, bytes);

        OpCode::ArithmeticFromToRegMem {
            op,
            dir,
            reg,
            reg_or_mem,
        }
    }
}

#[derive(Clone)]
pub struct ArithmeticImmediateToRegMemDecoder {}

impl ArithmeticImmediateToRegMemDecoder {
    const SIGN_EXTEND_MASK: u8 = 0b0000_0010;
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ArithmeticImmediateToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let sign_extend = code & Self::SIGN_EXTEND_MASK != 0;
        let width = decode_width(code, Self::WIDTH_MASK);

        let next = bytes.next().unwrap();

        let mode = decode_mode(*next >> 6 & 0b0000_0011);
        let op = decode_arithmetic_op((*next >> 3) & 0b0000_0111);
        let reg_or_mem = decode_reg_or_mem(*next & 0b0000_0111, mode, width, bytes);

        let data = if !sign_extend {
            decode_immediate(bytes, width)
        } else {
            decode_immediate(bytes, OpWidth::Byte)
        };

        OpCode::ArithmeticImmediateToRegMem {
            op,
            width,
            data,
            reg_or_mem,
        }
    }
}

#[derive(Clone)]
pub struct ArithmeticImmediateToAccumulatorDecoder {}

impl ArithmeticImmediateToAccumulatorDecoder {
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ArithmeticImmediateToAccumulatorDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let op = decode_arithmetic_op((code >> 3) & 0b0000_0111);
        let width = decode_width(code, Self::WIDTH_MASK);
        let data = decode_immediate(bytes, width);

        OpCode::ArithmeticImmediateToAccumulator { op, width, data }
    }
}

fn decode_arithmetic_op(byte: u8) -> ArithmeticOp {
    match byte {
        0 => ArithmeticOp::Add,
        5 => ArithmeticOp::Sub,
        7 => ArithmeticOp::Cmp,
        _ => todo!("not implemented yet"),
    }
}

fn effective_address_base(mem: u8) -> &'static str {
    match mem {
        0 => "bx + si",
        1 => "bx + di",
        2 => "bp + si",
        3 => "bp + di",
        4 => "si",
        5 => "di",
        6 => "bp",
        7 => "bx",
        _ => panic!("impossible"),
    }
}

fn decode_i8(lo: &u8) -> i16 {
    i8::from_le_bytes([*lo]) as i16
}

fn decode_i16(lo: &u8, hi: &u8) -> i16 {
    i16::from_le_bytes([*lo, *hi])
}

fn decode_immediate(bytes: &mut dyn Iterator<Item = &u8>, width: OpWidth) -> i16 {
    match width {
        OpWidth::Byte => decode_i8(bytes.next().unwrap()),
        OpWidth::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
    }
}

fn decode_dir(code: u8, dir_mask: u8) -> Direction {
    if code & dir_mask != 0 {
        Direction::ToRegister
    } else {
        Direction::FromRegister
    }
}

fn decode_width(code: u8, width_mask: u8) -> OpWidth {
    if code & width_mask != 0 {
        OpWidth::Word
    } else {
        OpWidth::Byte
    }
}

fn decode_mode(mode: u8) -> Mode {
    match mode {
        0 => Mode::MemoryNoDisplacement,
        1 => Mode::MemoryEightBitDisplacement,
        2 => Mode::MemorySixteenBitDisplacement,
        3 => Mode::Register,
        _ => panic!("impossible, we masked out exactly 2 bits"),
    }
}

fn decode_address(bytes: &mut dyn Iterator<Item = &u8>, width: OpWidth) -> i16 {
    match width {
        OpWidth::Byte => decode_i8(bytes.next().unwrap()),
        OpWidth::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
    }
}

fn decode_reg_or_mem(
    reg_or_mem: u8,
    mode: Mode,
    width: OpWidth,
    bytes: &mut dyn Iterator<Item = &u8>,
) -> String {
    match mode {
        Mode::Register => decode_reg(reg_or_mem, width).to_owned(),
        Mode::MemoryNoDisplacement if reg_or_mem == 6 => {
            let direct = match width {
                OpWidth::Byte => decode_i8(bytes.next().unwrap()),
                OpWidth::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
            };
            format!("[{}]", direct)
        }
        Mode::MemoryNoDisplacement => format!("[{}]", effective_address_base(reg_or_mem)),
        Mode::MemoryEightBitDisplacement => {
            let displacement = decode_i8(bytes.next().unwrap());
            format!(
                "[{}{}]",
                effective_address_base(reg_or_mem),
                format_displacement(displacement)
            )
        }
        Mode::MemorySixteenBitDisplacement => {
            let displacement = decode_i16(bytes.next().unwrap(), bytes.next().unwrap());
            format!(
                "[{}{}]",
                effective_address_base(reg_or_mem),
                format_displacement(displacement)
            )
        }
    }
}

fn decode_reg(reg: u8, width: OpWidth) -> &'static str {
    match width {
        OpWidth::Byte => match reg {
            0 => "al",
            1 => "cl",
            2 => "dl",
            3 => "bl",
            4 => "ah",
            5 => "ch",
            6 => "dh",
            7 => "bh",
            _ => panic!("impossible, we're only selecting 3 bits"),
        },
        OpWidth::Word => match reg {
            0 => "ax",
            1 => "cx",
            2 => "dx",
            3 => "bx",
            4 => "sp",
            5 => "bp",
            6 => "si",
            7 => "di",
            _ => panic!("impossible, we're only selecting 3 bits"),
        },
    }
}

fn format_displacement(displacement: i16) -> String {
    if displacement == 0 {
        "".to_string()
    } else if displacement > 0 {
        format!(" + {displacement}")
    } else if displacement == -256 {
        " - 256".to_string()
    } else {
        format!(" - {}", -displacement)
    }
}
