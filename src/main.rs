use std::path::Path;
use std::{env, fs};

mod lookup;
use lookup::OpDecoderLookup;

mod ops;
use ops::*;

mod decode;
use decode::*;

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
    let mut iter = bytes.iter().enumerate().peekable();
    println!("bits 16");
    loop {
        let byte = iter.next();
        if byte.is_none() {
            break;
        }
        let (i, byte) = byte.unwrap();

        let decoder = lookup
            .get(byte)
            .expect(&format!("no decoder found for {byte:#b}"));

        let code = decoder.decode(*byte, &mut iter.by_ref().map(|(_i, byte)| byte));
        let next_i = match iter.peek() {
            Some((i, _byte)) => *i,
            None => bytes.len(),
        };

        println!("label_{i}:");
        println!("{}", code.encode(next_i));
    }
    Ok(())
}
