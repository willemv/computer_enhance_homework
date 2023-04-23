use std::collections::HashMap;
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
enum OpSize {
    Byte,
    Word,
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
    Mov {
        dir: Direction,
        reg: &'static str,
        reg_or_mem: String,
    },
    ImmediateMovRegMem {
        size: OpSize,
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
}

impl OpCode {
    fn encode(&self) -> String {
        match *self {
            OpCode::Mov {
                dir,
                reg,
                ref reg_or_mem,
            } => match dir {
                Direction::FromRegister => format!("mov {reg_or_mem}, {reg}"),
                Direction::ToRegister => format!("mov {reg}, {reg_or_mem}"),
            },
            OpCode::ImmediateMovRegMem {
                size,
                ref reg_or_mem,
                data,
            } => {
                format!(
                    "mov {reg_or_mem}, {} {data}",
                    match size {
                        OpSize::Byte => "byte",
                        OpSize::Word => "word",
                    }
                )
            }
            OpCode::ImmediateMovReg { reg, data } => {
                format!("mov {reg}, {data}")
            }
            OpCode::AccumulatorMove { dir, addr } => match dir {
                Direction::FromRegister => format!("mov [{addr}], ax"),
                Direction::ToRegister => format!("mov ax, [{addr}]"),
            },
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
    const SIZE_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for MovDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let dir = if code & MovDecoder::DIR_MASK != 0 {
            Direction::ToRegister
        } else {
            Direction::FromRegister
        };
        let size = if code & MovDecoder::SIZE_MASK != 0 {
            OpSize::Word
        } else {
            OpSize::Byte
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

        let reg = decode_reg((*next >> 3) & 0b0000_0111, size);
        let reg_or_mem = next & 0b0000_0111;

        let reg_or_mem = match mode {
            Mode::Register => decode_reg(reg_or_mem, size).to_owned(),
            Mode::MemoryNoDisplacement if reg_or_mem == 6 => {
                let direct = match size {
                    OpSize::Byte => decode_i8(bytes.next().unwrap()),
                    OpSize::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
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
        };

        OpCode::Mov {
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
    const SIZE_MASK: u8 = 0b0000_0001;
}

impl OpCodeDecoder for ImmediateMovToRegMemDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let size = if code & Self::SIZE_MASK != 0 {
            OpSize::Word
        } else {
            OpSize::Byte
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
            Mode::Register => decode_reg(reg_or_mem, size).to_owned(),
            Mode::MemoryNoDisplacement if reg_or_mem == 6 => {
                let direct = match size {
                    OpSize::Byte => decode_i8(bytes.next().unwrap()),
                    OpSize::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
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

        let data: i16 = match size {
            OpSize::Byte => decode_i8(bytes.next().unwrap()),
            OpSize::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
        };

        OpCode::ImmediateMovRegMem {
            size,
            reg_or_mem,
            data,
        }
    }
}

#[derive(Clone)]
struct ImmediateMovToRegDecoder {}

impl ImmediateMovToRegDecoder {
    const SIZE_MASK: u8 = 0b0000_1000;
}

impl OpCodeDecoder for ImmediateMovToRegDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let size = if code & Self::SIZE_MASK != 0 {
            OpSize::Word
        } else {
            OpSize::Byte
        };

        let reg = code & 0b0000_0111;
        let reg = decode_reg(reg, size);

        let data: i16 = match size {
            OpSize::Byte => decode_i8(bytes.next().unwrap()),
            OpSize::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
        };

        OpCode::ImmediateMovReg { reg, data }
    }
}

#[derive(Clone)]
struct MovAccumulatorDecoder {}

impl MovAccumulatorDecoder {
    const SIZE_MASK: u8 = 0b0000_0001;
    const DIR_MASK: u8 = 0b0000_0010;
}
impl OpCodeDecoder for MovAccumulatorDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode {
        let size = if code & Self::SIZE_MASK != 0 {
            OpSize::Word
        } else {
            OpSize::Byte
        };
        let dir = if code & Self::DIR_MASK != 0 {
            Direction::FromRegister
        } else {
            Direction::ToRegister
        };

        let address: i16 = match size {
            OpSize::Byte => decode_i8(bytes.next().unwrap()),
            OpSize::Word => decode_i16(bytes.next().unwrap(), bytes.next().unwrap()),
        };

        OpCode::AccumulatorMove { dir, addr: address }
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

fn decode_reg(reg: u8, size: OpSize) -> &'static str {
    match size {
        OpSize::Byte => match reg {
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
        OpSize::Word => match reg {
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
