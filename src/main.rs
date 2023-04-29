use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        std::process::exit(1);
    }

    let exit_code = match encode_to_assembler(&args[1]) {
        Ok(_) => 0,
        Err(e) => {
            println!("Error converting to assembler: {e}");
            1
        }
    };
    std::process::exit(exit_code);
}

struct OpDecoderLookup {
    map: HashMap<u8, Box<dyn OpCodeDecoder>>,
}

impl OpDecoderLookup {
    fn new() -> OpDecoderLookup {
        OpDecoderLookup {
            map: HashMap::new(),
        }
    }

    fn insert<D: OpCodeDecoder + Clone + 'static>(&mut self, pattern: &str, decoder: D) {
        let pattern = pattern.strip_prefix("0b").unwrap_or(pattern);
        let pattern = pattern.replace('_', "");

        let mut v = vec![];
        Self::expand(&pattern, &mut v);

        for b in v {
            self.map.insert(b, Box::new(decoder.clone()));
        }
    }

    fn get(&self, opcode: &u8) -> Option<&Box<dyn OpCodeDecoder>> {
        self.map.get(opcode)
    }

    fn expand(i: &str, v: &mut Vec<u8>) {
        if i.chars().all(|c| c == '1' || c == '0') {
            let p = u8::from_str_radix(i, 2).expect(&format!("could not parse {i}"));
            v.push(p);
        } else {
            Self::expand(&i.replacen(|c| !char::is_numeric(c), "0", 1), v);
            Self::expand(&i.replacen(|c| !char::is_numeric(c), "1", 1), v);
        }
    }
}

fn encode_to_assembler<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut l = OpDecoderLookup::new();
    l.insert("0b1000_10dw", MovDecoder {});
    l.insert("0b1100_011w", ImmediateMovToRegMemDecoder {});
    l.insert("0b1011_wreg", ImmediateMovToRegDecoder {});
    l.insert("0b1010_00dw", MovAccumulatorDecoder {});

    l.insert("0b00xx_x0dw", ArithmeticFromToRegMemDecoder {});
    l.insert("0b1000_00sw", ArithmeticImmediateToRegMemDecoder {});
    l.insert("0b00xx_x1dw", ArithmeticImmediateToAccumulatorDecoder {});

    l.insert("0b0111_0100", JumpDecoder::new(OpCode::JumpOnEqual));
    l.insert("0b0111_1100", JumpDecoder::new(OpCode::JumpOnLess));
    l.insert("0b0111_1110", JumpDecoder::new(OpCode::JumpOnNotGreater));
    l.insert("0b0111_0010", JumpDecoder::new(OpCode::JumpOnBelow));
    l.insert("0b0111_0110", JumpDecoder::new(OpCode::JumpOnNotAbove));
    l.insert("0b0111_1010", JumpDecoder::new(OpCode::JumpOnParity));
    l.insert("0b0111_0000", JumpDecoder::new(OpCode::JumpOnOverflow));
    l.insert("0b0111_1000", JumpDecoder::new(OpCode::JumpOnSign));
    l.insert("0b0111_0101", JumpDecoder::new(OpCode::JumpOnNotEqual));
    l.insert("0b0111_1101", JumpDecoder::new(OpCode::JumpOnNotLess));
    l.insert("0b0111_1111", JumpDecoder::new(OpCode::JumpOnGreater));
    l.insert("0b0111_0011", JumpDecoder::new(OpCode::JumpOnNotBelow));
    l.insert("0b0111_0111", JumpDecoder::new(OpCode::JumpOnAbove));
    l.insert("0b0111_1011", JumpDecoder::new(OpCode::JumpOnNoParity));
    l.insert("0b0111_0001", JumpDecoder::new(OpCode::JumpOnNoOverflow));
    l.insert("0b0111_1001", JumpDecoder::new(OpCode::JumpOnNotSign));
    l.insert("0b1110_0010", JumpDecoder::new(OpCode::Loop));
    l.insert("0b1110_0001", JumpDecoder::new(OpCode::LoopWhileEqual));
    l.insert("0b1110_0000", JumpDecoder::new(OpCode::LoopWhileNotEqual));
    l.insert("0b1110_0011", JumpDecoder::new(OpCode::JumpOnCxZero));

    let lookup = l;

    let bytes = fs::read(path)?;
    let mut iter = bytes.iter();
    println!("bits 16");
    loop {
        let byte = iter.next();
        if byte.is_none() {
            break;
        }
        let byte = byte.unwrap();

        let decoder = lookup
            .get(byte)
            .expect(&format!("no decoder found for {byte:#b}"));
        let code = decoder.decode(*byte, &mut iter);
        println!("{}", code.encode());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    ToRegister,
    FromRegister,
}

#[derive(Debug, Clone, Copy)]
enum OpWidth {
    Byte,
    Word,
}

impl Display for OpWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OpWidth::Byte => f.write_str("byte"),
            OpWidth::Word => f.write_str("word"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    MemoryNoDisplacement,
    MemoryEightBitDisplacement,
    MemorySixteenBitDisplacement,
    Register,
}

#[derive(Debug, Clone)]
enum OpCode {
    MovToFromRegMem {
        dir: Direction,
        reg: &'static str,
        reg_or_mem: String,
    },
    ImmediateMovRegMem {
        width: OpWidth,
        reg_or_mem: String,
        data: i16,
    },
    ImmediateMovReg {
        reg: &'static str,
        data: i16,
    },
    AccumulatorMove {
        dir: Direction,
        addr: i16,
    },
    ArithmeticFromToRegMem {
        op: ArithmeticOp,
        dir: Direction,
        reg: &'static str,
        reg_or_mem: String,
    },
    ArithmeticImmediateToRegMem {
        op: ArithmeticOp,
        width: OpWidth,
        data: i16,
        reg_or_mem: String,
    },
    ArithmeticImmediateToAccumulator {
        op: ArithmeticOp,
        width: OpWidth,
        data: i16,
    },
    JumpOnEqual(i8),
    JumpOnLess(i8),
    JumpOnNotGreater(i8),
    JumpOnBelow(i8),
    JumpOnNotAbove(i8),
    JumpOnParity(i8),
    JumpOnOverflow(i8),
    JumpOnSign(i8),
    JumpOnNotEqual(i8),
    JumpOnNotLess(i8),
    JumpOnGreater(i8),
    JumpOnNotBelow(i8),
    JumpOnAbove(i8),
    JumpOnNoParity(i8),
    JumpOnNoOverflow(i8),
    JumpOnNotSign(i8),
    Loop(i8),
    LoopWhileEqual(i8),
    LoopWhileNotEqual(i8),
    JumpOnCxZero(i8),
}

impl OpCode {
    fn encode(&self) -> String {
        match *self {
            OpCode::MovToFromRegMem {
                dir,
                reg,
                ref reg_or_mem,
            } => match dir {
                Direction::FromRegister => format!("mov {reg_or_mem}, {reg}"),
                Direction::ToRegister => format!("mov {reg}, {reg_or_mem}"),
            },
            OpCode::ImmediateMovRegMem {
                width,
                ref reg_or_mem,
                data,
            } => {
                format!("mov {reg_or_mem}, {width} {data}")
            }
            OpCode::ImmediateMovReg { reg, data } => {
                format!("mov {reg}, {data}")
            }
            OpCode::AccumulatorMove { dir, addr } => match dir {
                Direction::FromRegister => format!("mov [{addr}], ax"),
                Direction::ToRegister => format!("mov ax, [{addr}]"),
            },
            OpCode::ArithmeticFromToRegMem {
                op,
                dir,
                reg,
                ref reg_or_mem,
            } => match dir {
                Direction::FromRegister => format!("{op} {reg_or_mem}, {reg}"),
                Direction::ToRegister => format!("{op} {reg}, {reg_or_mem}"),
            },
            OpCode::ArithmeticImmediateToRegMem {
                op,
                width,
                data,
                ref reg_or_mem,
            } => {
                format!("{op} {reg_or_mem}, {width} {data}")
            }
            OpCode::ArithmeticImmediateToAccumulator { op, width, data } => {
                format!(
                    "{op} {}, {data}",
                    match width {
                        OpWidth::Byte => "al",
                        OpWidth::Word => "ax",
                    }
                )
            }
            OpCode::JumpOnEqual(disp) => format!("je {disp}"),
            OpCode::JumpOnLess(disp) => format!("jl {disp}"),
            OpCode::JumpOnNotGreater(disp) => format!("jle {disp}"),
            OpCode::JumpOnBelow(disp) => format!("jb {disp}"),
            OpCode::JumpOnNotAbove(disp) => format!("jbe {disp}"),
            OpCode::JumpOnParity(disp) => format!("jp {disp}"),
            OpCode::JumpOnOverflow(disp) => format!("jo {disp}"),
            OpCode::JumpOnSign(disp) => format!("js {disp}"),
            OpCode::JumpOnNotEqual(disp) => format!("jne {disp}"),
            OpCode::JumpOnNotLess(disp) => format!("jnl {disp}"),
            OpCode::JumpOnGreater(disp) => format!("jg {disp}"),
            OpCode::JumpOnNotBelow(disp) => format!("jnb {disp}"),
            OpCode::JumpOnAbove(disp) => format!("jnbe {disp}"),
            OpCode::JumpOnNoParity(disp) => format!("jnp {disp}"),
            OpCode::JumpOnNoOverflow(disp) => format!("jno {disp}"),
            OpCode::JumpOnNotSign(disp) => format!("jns {disp}"),
            OpCode::Loop(disp) => format!("loop {disp}"),
            OpCode::LoopWhileEqual(disp) => format!("loope {disp}"),
            OpCode::LoopWhileNotEqual(disp) => format!("loopne {disp}"),
            OpCode::JumpOnCxZero(disp) => format!("jcxz {disp}"),
        }
    }
}

trait OpCodeDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode;
}

#[derive(Clone, Copy)]
struct MovDecoder {}

impl MovDecoder {
    const DIR_MASK: u8 = 0b0000_0010;
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for MovDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let dir = if code & MovDecoder::DIR_MASK != 0 {
            Direction::ToRegister
        } else {
            Direction::FromRegister
        };
        let width = if code & MovDecoder::WIDTH_MASK != 0 {
            OpWidth::Word
        } else {
            OpWidth::Byte
        };

        let next = bytes.next().unwrap();
        let mode = (*next >> 6) & 0b0000_0011;
        let mode = match mode {
            0 => Mode::MemoryNoDisplacement,
            1 => Mode::MemoryEightBitDisplacement,
            2 => Mode::MemorySixteenBitDisplacement,
            3 => Mode::Register,
            _ => panic!("impossible, we masked out exactly 2 bits"),
        };

        let reg = decode_reg((*next >> 3) & 0b0000_0111, width);
        let reg_or_mem = next & 0b0000_0111;

        let reg_or_mem = match mode {
            Mode::Register => decode_reg(reg_or_mem, width).to_owned(),
            Mode::MemoryNoDisplacement if reg_or_mem == 6 => {
                let direct = decode_immediate(bytes, width);
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
        };

        OpCode::MovToFromRegMem {
            dir,
            reg,
            reg_or_mem,
        }
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

#[derive(Clone)]
struct ImmediateMovToRegMemDecoder {}

impl ImmediateMovToRegMemDecoder {
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ImmediateMovToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let width = if code & Self::WIDTH_MASK != 0 {
            OpWidth::Word
        } else {
            OpWidth::Byte
        };

        let next = bytes.next().unwrap();
        let mode = (*next >> 6) & 0b0000_0011;
        let mode = match mode {
            0 => Mode::MemoryNoDisplacement,
            1 => Mode::MemoryEightBitDisplacement,
            2 => Mode::MemorySixteenBitDisplacement,
            3 => Mode::Register,
            _ => panic!("impossible, we masked out exactly 2 bits"),
        };

        let reg_or_mem = next & 0b0000_0111;

        let reg_or_mem = match mode {
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
                let displacement = bytes.next().unwrap();

                if displacement == &0 {
                    format!("[{}]", effective_address_base(reg_or_mem))
                } else {
                    format!(
                        "[{} + {}]",
                        effective_address_base(reg_or_mem),
                        displacement
                    )
                }
            }
            Mode::MemorySixteenBitDisplacement => {
                let displacement = decode_i16(bytes.next().unwrap(), bytes.next().unwrap());
                if displacement == 0 {
                    format!("[{}]", effective_address_base(reg_or_mem))
                } else {
                    format!(
                        "[{} + {}]",
                        effective_address_base(reg_or_mem),
                        displacement
                    )
                }
            }
        };

        let data: i16 = match width {
            OpWidth::Byte => decode_i8(bytes.next().unwrap()),
            OpWidth::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
        };

        OpCode::ImmediateMovRegMem {
            width,
            reg_or_mem,
            data,
        }
    }
}

#[derive(Clone)]
struct ImmediateMovToRegDecoder {}

impl ImmediateMovToRegDecoder {
    const WIDTH_MASK: u8 = 0b0000_1000;
}

impl OpCodeDecoder for ImmediateMovToRegDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let width = if code & Self::WIDTH_MASK != 0 {
            OpWidth::Word
        } else {
            OpWidth::Byte
        };

        let reg = code & 0b0000_0111;
        let reg = decode_reg(reg, width);

        let data: i16 = match width {
            OpWidth::Byte => decode_i8(bytes.next().unwrap()),
            OpWidth::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
        };

        OpCode::ImmediateMovReg { reg, data }
    }
}

#[derive(Clone)]
struct MovAccumulatorDecoder {}

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

#[derive(Debug, Clone, Copy)]
enum ArithmeticOp {
    Add,
    Sub,
    Cmp,
}

impl Display for ArithmeticOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ArithmeticOp::Add => f.write_str("add"),
            ArithmeticOp::Sub => f.write_str("sub"),
            ArithmeticOp::Cmp => f.write_str("cmp"),
        }
    }
}

#[derive(Clone)]
struct ArithmeticFromToRegMemDecoder {}

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
struct ArithmeticImmediateToRegMemDecoder {}

impl ArithmeticImmediateToRegMemDecoder {
    const SIGN_EXTEND_MASK: u8 = 0b0000_0010;
    const WIDTH_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ArithmeticImmediateToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let next = bytes.next().unwrap();

        let sign_extend = code & Self::SIGN_EXTEND_MASK != 0;
        let width = decode_width(code, Self::WIDTH_MASK);

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

fn decode_arithmetic_op(byte: u8) -> ArithmeticOp {
    match byte {
        0 => ArithmeticOp::Add,
        5 => ArithmeticOp::Sub,
        7 => ArithmeticOp::Cmp,
        _ => todo!("not implemented yet"),
    }
}

#[derive(Clone)]
struct ArithmeticImmediateToAccumulatorDecoder {}

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

#[derive(Clone)]
struct JumpDecoder {
    jump_op: fn(i8) -> OpCode,
}

impl JumpDecoder {
    fn new(op: fn(i8) -> OpCode) -> JumpDecoder {
        JumpDecoder { jump_op: op }
    }
}

impl OpCodeDecoder for JumpDecoder {
    fn decode(&self, _code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let disp = i8::from_le_bytes([*bytes.next().unwrap()]);
        (self.jump_op)(disp)
    }
}
