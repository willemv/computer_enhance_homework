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

fn encode_to_assembler<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let mut lookup: HashMap<u8, Box<dyn OpCodeDecoder>> = HashMap::new();
    lookup.insert(0b1000_1000, Box::new(MovDecoder {}));
    lookup.insert(0b1000_1001, Box::new(MovDecoder {}));
    lookup.insert(0b1000_1010, Box::new(MovDecoder {}));
    lookup.insert(0b1000_1011, Box::new(MovDecoder {}));

    let lookup = lookup;

    let bytes = fs::read(path)?;
    let mut iter = bytes.iter();
    println!("bits 16");
    loop {
        let byte = iter.next();
        if byte.is_none() {
            break;
        }
        let byte = byte.unwrap();

        let decoder = lookup.get(byte).unwrap();
        let code = decoder.decode(byte.clone(), &mut iter);
        match code {
            OpCode::MOV {
                dir,
                reg,
                reg_or_mem,
            } => {
                match dir {
                    Direction::FromRegister => println!("mov {reg_or_mem}, {reg}"),
                    Direction::ToRegister => println!("mov {reg}, {reg_or_mem}")
                }
            }
        }
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
    Memory,
    MemoryEightBitDisplacement,
    MemorySixteenBitDisplacement,
    Register,
}

#[derive(Debug, Clone, Copy)]
enum OpCode {
    MOV {
        dir: Direction,
        reg: &'static str,
        reg_or_mem: &'static str,
    },
}

trait OpCodeDecoder {
    fn decode(&self, code: u8, bytes: &mut dyn Iterator<Item = &u8>) -> OpCode;
}

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
        let mode = (next.clone() >> 6) & 0b0000_0011;
        let mode = match mode {
            0 => Mode::Memory,
            1 => Mode::MemoryEightBitDisplacement,
            2 => Mode::MemorySixteenBitDisplacement,
            3 => Mode::Register,
            _ => panic!("impossible, we masked out exactly 2 bits"),
        };

        let reg = decode_reg((next.clone() >> 3) & 0b0000_0111, size);
        let reg_or_mem = match mode {
            Mode::Register => decode_reg(next & 0b0000_0111, size),
            _ => todo!("not part of homework assignment yet"),
        };

        OpCode::MOV {
            dir,
            reg,
            reg_or_mem,
        }
    }
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
