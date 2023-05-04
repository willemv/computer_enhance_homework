use crate::decode::*;
use crate::lookup::*;
use crate::ops::Instruction;

pub struct Decoder {
    lookup: OpDecoderLookup,
}

impl Decoder {
    pub fn new() -> Decoder {
        let mut l = OpDecoderLookup::new();
        l.insert("0b1000_10dw", MovToFromRegMemDecoder {});
        l.insert("0b1100_011w", ImmediateMovToRegMemDecoder {});
        l.insert("0b1011_wreg", ImmediateMovToRegDecoder {});
        l.insert("0b1010_00dw", MovAccumulatorDecoder {});

        l.insert("0b00xx_x0dw", ArithmeticFromToRegMemDecoder {});
        l.insert("0b1000_00sw", ArithmeticImmediateToRegMemDecoder {});
        l.insert("0b00xx_x1dw", ArithmeticImmediateToAccumulatorDecoder {});

        l.insert("0b0111_0100", JumpDecoder::new(Instruction::JumpOnEqual));
        l.insert("0b0111_1100", JumpDecoder::new(Instruction::JumpOnLess));
        l.insert("0b0111_1110", JumpDecoder::new(Instruction::JumpOnNotGreater));
        l.insert("0b0111_0010", JumpDecoder::new(Instruction::JumpOnBelow));
        l.insert("0b0111_0110", JumpDecoder::new(Instruction::JumpOnNotAbove));
        l.insert("0b0111_1010", JumpDecoder::new(Instruction::JumpOnParity));
        l.insert("0b0111_0000", JumpDecoder::new(Instruction::JumpOnOverflow));
        l.insert("0b0111_1000", JumpDecoder::new(Instruction::JumpOnSign));
        l.insert("0b0111_0101", JumpDecoder::new(Instruction::JumpOnNotEqual));
        l.insert("0b0111_1101", JumpDecoder::new(Instruction::JumpOnNotLess));
        l.insert("0b0111_1111", JumpDecoder::new(Instruction::JumpOnGreater));
        l.insert("0b0111_0011", JumpDecoder::new(Instruction::JumpOnNotBelow));
        l.insert("0b0111_0111", JumpDecoder::new(Instruction::JumpOnAbove));
        l.insert("0b0111_1011", JumpDecoder::new(Instruction::JumpOnNoParity));
        l.insert("0b0111_0001", JumpDecoder::new(Instruction::JumpOnNoOverflow));
        l.insert("0b0111_1001", JumpDecoder::new(Instruction::JumpOnNotSign));
        l.insert("0b1110_0010", JumpDecoder::new(Instruction::Loop));
        l.insert("0b1110_0001", JumpDecoder::new(Instruction::LoopWhileEqual));
        l.insert("0b1110_0000", JumpDecoder::new(Instruction::LoopWhileNotEqual));
        l.insert("0b1110_0011", JumpDecoder::new(Instruction::JumpOnCxZero));

        let lookup = l;

        Decoder { lookup }
    }

    pub fn decode_next(&self, iter: &mut dyn Iterator<Item = &u8>) -> Option<Instruction> {
        let byte = iter.next();
        if byte.is_none() {
            return None;
        }

        let byte = byte.unwrap();

        let decoder = self.lookup.get(byte).expect(&format!("no decoder found for {byte:#b}"));

        let code = decoder.decode(*byte, iter);

        Some(code)
    }
}
