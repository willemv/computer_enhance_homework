use std::fmt::{Display, Formatter};

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u16 {
        const Carry = 0b0000_0000_0001;
        const Parity = 0b0000_0000_0100;
        const AuxiliaryCarry = 0b0000_0001_0000;
        const Zero = 0b0000_0100_0000;
        const Sign = 0b0000_1000_0000;
        const Overflow = 0b1000_0000_0000;
    }
}

impl Flags {
    pub fn arithmetic_flags() -> Flags {
        Flags::Zero | Flags::Parity | Flags::Carry | Flags::Sign | Flags::AuxiliaryCarry | Flags::Overflow
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        if self.contains(Flags::Carry) { f.write_str("C")?; }
        if self.contains(Flags::Parity) { f.write_str("P")?; }
        if self.contains(Flags::AuxiliaryCarry) { f.write_str("A")?; }
        if self.contains(Flags::Zero) { f.write_str("Z")?; }
        if self.contains(Flags::Sign) { f.write_str("S")?; }
        if self.contains(Flags::Overflow) { f.write_str("O")?; }

        Ok(())
    }
}

