use std::fmt::{Display, Formatter};

use bitflags::bitflags;
use bitflags::parser;
use parser::to_writer;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u16 {
        const S = 0b1000_0000;
        const Z = 0b0100_0000;
        const A = 0b0001_0000;
        const P = 0b0000_0100;
        const C = 0b0000_0001;
        const O = 0b1000_0000_0000;
    }
}

impl Flags {
    pub fn arithmetic_flags() -> Flags {
        Flags::Z | Flags::P | Flags::C | Flags::S | Flags::A | Flags::O
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        to_writer(self, f)
    }
}

