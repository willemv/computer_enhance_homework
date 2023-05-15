use crate::decode::*;
use crate::lookup::*;
use crate::ops::Instruction;

pub struct Decoder {
    lookup: OpDecoderLookup,
}

impl Decoder {
    pub fn new() -> Decoder {
        let mut lookup = OpDecoderLookup::new();
        lookup.insert("0b1000_10dw", MovToFromRegMemDecoder {});
        lookup.insert("0b1100_011w", ImmediateMovToRegMemDecoder {});
        lookup.insert("0b1011_wreg", ImmediateMovToRegDecoder {});
        lookup.insert("0b1010_00dw", MovAccumulatorDecoder {});
        lookup.insert("0b1000_11d0", MovSegmentDecoder {});

        lookup.insert("0b00xx_x0dw", ArithmeticFromToRegMemDecoder {});
        lookup.insert("0b1000_00sw", ArithmeticImmediateToRegMemDecoder {});
        lookup.insert("0b00xx_x1dw", ArithmeticImmediateToAccumulatorDecoder {});

        lookup.insert("0b0111_0100", JumpDecoder::new(Instruction::JumpOnEqual));
        lookup.insert("0b0111_1100", JumpDecoder::new(Instruction::JumpOnLess));
        lookup.insert("0b0111_1110", JumpDecoder::new(Instruction::JumpOnNotGreater));
        lookup.insert("0b0111_0010", JumpDecoder::new(Instruction::JumpOnBelow));
        lookup.insert("0b0111_0110", JumpDecoder::new(Instruction::JumpOnNotAbove));
        lookup.insert("0b0111_1010", JumpDecoder::new(Instruction::JumpOnParity));
        lookup.insert("0b0111_0000", JumpDecoder::new(Instruction::JumpOnOverflow));
        lookup.insert("0b0111_1000", JumpDecoder::new(Instruction::JumpOnSign));
        lookup.insert("0b0111_0101", JumpDecoder::new(Instruction::JumpOnNotEqual));
        lookup.insert("0b0111_1101", JumpDecoder::new(Instruction::JumpOnNotLess));
        lookup.insert("0b0111_1111", JumpDecoder::new(Instruction::JumpOnGreater));
        lookup.insert("0b0111_0011", JumpDecoder::new(Instruction::JumpOnNotBelow));
        lookup.insert("0b0111_0111", JumpDecoder::new(Instruction::JumpOnAbove));
        lookup.insert("0b0111_1011", JumpDecoder::new(Instruction::JumpOnNoParity));
        lookup.insert("0b0111_0001", JumpDecoder::new(Instruction::JumpOnNoOverflow));
        lookup.insert("0b0111_1001", JumpDecoder::new(Instruction::JumpOnNotSign));
        lookup.insert("0b1110_0010", JumpDecoder::new(Instruction::Loop));
        lookup.insert("0b1110_0001", JumpDecoder::new(Instruction::LoopWhileEqual));
        lookup.insert("0b1110_0000", JumpDecoder::new(Instruction::LoopWhileNotEqual));
        lookup.insert("0b1110_0011", JumpDecoder::new(Instruction::JumpOnCxZero));

        Decoder { lookup }
    }

    pub fn decode_next(&self, iter: &mut dyn Iterator<Item = &u8>) -> Option<Instruction> {
        let byte = iter.next()?;

        let decoder = self.lookup.get(byte).unwrap_or_else(|| panic!("no decoder found for {byte:#b}"));

        let code = decoder.decode(*byte, iter);

        Some(code)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}