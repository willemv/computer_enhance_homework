use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Register {
    A,
    C,
    D,
    B,
    Sp,
    Bp,
    Si,
    Di,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SegmentRegister {
    Es,
    Cs,
    Ss,
    Ds,
}

impl Display for SegmentRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterAccess {
    pub reg: Register,
    pub width: OpWidth,
    pub offset: u8,
}

impl RegisterAccess {
    pub fn new(reg: Register, width: OpWidth, offset: u8) -> RegisterAccess {
        RegisterAccess { reg, width, offset }
    }

    fn encode(&self) -> String {
        use Register::*;

        match self.reg {
            A | C | D | B => format!(
                "{}{}",
                self.reg,
                match self.width {
                    OpWidth::Word => "x",
                    OpWidth::Byte =>
                        if self.offset == 0 {
                            "l"
                        } else {
                            "h"
                        },
                }
            ),
            Sp | Bp | Si | Di => format!("{}", self.reg),
        }
    }
}

impl Display for RegisterAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EffectiveAddressBase {
    Direct,
    BxPlusSi,
    BxPlusDi,
    BpPlusSi,
    BpPlusDi,
    Si,
    Di,
    Bp,
    Bx,
}

impl Display for EffectiveAddressBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => panic!(),
            Self::BxPlusSi => f.write_str("bx + si"),
            Self::BxPlusDi => f.write_str("bx + di"),
            Self::BpPlusSi => f.write_str("bp + si"),
            Self::BpPlusDi => f.write_str("bp + di"),
            Self::Si => f.write_str("si"),
            Self::Di => f.write_str("di"),
            Self::Bp => f.write_str("bp"),
            Self::Bx => f.write_str("bx"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EffectiveAddress {
    pub base: EffectiveAddressBase,
    pub displacement: i16,
}

#[derive(Debug, Clone, Copy)]
pub enum RegOrMem {
    Reg(RegisterAccess),
    Mem(EffectiveAddress),
}

impl Display for RegOrMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegOrMem::Reg(ra) => ra.fmt(f),

            RegOrMem::Mem(EffectiveAddress { base, displacement }) if matches!(base, &EffectiveAddressBase::Direct) => {
                f.write_fmt(format_args!("[{displacement}]"))
            }
            RegOrMem::Mem(EffectiveAddress { base, displacement }) => {
                if displacement == &0 {
                    f.write_fmt(format_args!("[{base}]"))
                } else if displacement > &0 {
                    f.write_fmt(format_args!("[{base} + {displacement}]"))
                } else if displacement == &(-256) {
                    f.write_fmt(format_args!("[{base} - 256]"))
                } else {
                    f.write_fmt(format_args!("[{base} - {}]", -(*displacement)))
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    ToRegister,
    FromRegister,
}

#[derive(Debug, Clone, Copy)]
pub enum OpWidth {
    Byte,
    Word,
}

impl std::fmt::Display for OpWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OpWidth::Byte => f.write_str("byte"),
            OpWidth::Word => f.write_str("word"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOp {
    Add,
    Adc,
    Sub,
    Sbb,
    Cmp,
}

impl Display for ArithmeticOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ArithmeticOp::Add => f.write_str("add"),
            ArithmeticOp::Adc => f.write_str("adc"),
            ArithmeticOp::Sub => f.write_str("sub"),
            ArithmeticOp::Sbb => f.write_str("sbb"),
            ArithmeticOp::Cmp => f.write_str("cmp"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    MovToFromRegMem {
        dir: Direction,
        reg: RegisterAccess,
        reg_or_mem: RegOrMem,
    },
    ImmediateMovRegMem {
        width: OpWidth,
        reg_or_mem: RegOrMem,
        data: i16,
    },
    ImmediateMovReg {
        reg: RegisterAccess,
        data: i16,
    },
    AccumulatorMove {
        dir: Direction,
        addr: i16,
    },
    SegmentRegisterMove {
        dir: Direction,
        seg_reg: SegmentRegister,
        reg_or_mem: RegOrMem,
    },
    ArithmeticFromToRegMem {
        op: ArithmeticOp,
        dir: Direction,
        width: OpWidth,
        reg: RegisterAccess,
        reg_or_mem: RegOrMem,
    },
    ArithmeticImmediateToRegMem {
        op: ArithmeticOp,
        width: OpWidth,
        data: i16,
        reg_or_mem: RegOrMem,
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

impl Instruction {
    pub fn encode<F>(&self, format_jump: F) -> String
    where
        F: Fn(i8) -> String,
    {
        match *self {
            Instruction::MovToFromRegMem { dir, reg, ref reg_or_mem } => match dir {
                Direction::FromRegister => format!("mov {reg_or_mem}, {reg}"),
                Direction::ToRegister => format!("mov {reg}, {reg_or_mem}"),
            },
            Instruction::ImmediateMovRegMem {
                width,
                ref reg_or_mem,
                data,
            } => {
                format!("mov {reg_or_mem}, {width} {data}")
            }
            Instruction::ImmediateMovReg { reg, data } => {
                format!("mov {reg}, {data}")
            }
            Instruction::AccumulatorMove { dir, addr } => match dir {
                Direction::FromRegister => format!("mov [{addr}], ax"),
                Direction::ToRegister => format!("mov ax, [{addr}]"),
            },
            Instruction::SegmentRegisterMove { dir, seg_reg, reg_or_mem } => match dir {
                Direction::FromRegister => format!("mov {reg_or_mem}, {seg_reg}"),
                Direction::ToRegister => format!("mov {seg_reg}, {reg_or_mem}"),
            },
            Instruction::ArithmeticFromToRegMem {
                op,
                dir,
                width: _,
                reg,
                ref reg_or_mem,
            } => match dir {
                Direction::FromRegister => format!("{op} {reg_or_mem}, {reg}"),
                Direction::ToRegister => format!("{op} {reg}, {reg_or_mem}"),
            },
            Instruction::ArithmeticImmediateToRegMem {
                op,
                width,
                data,
                ref reg_or_mem,
            } => {
                format!("{op} {reg_or_mem}, {width} {data}")
            }
            Instruction::ArithmeticImmediateToAccumulator { op, width, data } => {
                format!(
                    "{op} {}, {data}",
                    match width {
                        OpWidth::Byte => "al",
                        OpWidth::Word => "ax",
                    }
                )
            }
            Instruction::JumpOnEqual(disp) => format!("je {}", format_jump(disp)),
            Instruction::JumpOnLess(disp) => format!("jl {}", format_jump(disp)),
            Instruction::JumpOnNotGreater(disp) => format!("jle {}", format_jump(disp)),
            Instruction::JumpOnBelow(disp) => format!("jb {}", format_jump(disp)),
            Instruction::JumpOnNotAbove(disp) => format!("jbe {}", format_jump(disp)),
            Instruction::JumpOnParity(disp) => format!("jp {}", format_jump(disp)),
            Instruction::JumpOnOverflow(disp) => format!("jo {}", format_jump(disp)),
            Instruction::JumpOnSign(disp) => format!("js {}", format_jump(disp)),
            Instruction::JumpOnNotEqual(disp) => format!("jne {}", format_jump(disp)),
            Instruction::JumpOnNotLess(disp) => format!("jnl {}", format_jump(disp)),
            Instruction::JumpOnGreater(disp) => format!("jg {}", format_jump(disp)),
            Instruction::JumpOnNotBelow(disp) => format!("jnb {}", format_jump(disp)),
            Instruction::JumpOnAbove(disp) => format!("jnbe {}", format_jump(disp)),
            Instruction::JumpOnNoParity(disp) => format!("jnp {}", format_jump(disp)),
            Instruction::JumpOnNoOverflow(disp) => format!("jno {}", format_jump(disp)),
            Instruction::JumpOnNotSign(disp) => format!("jns {}", format_jump(disp)),
            Instruction::Loop(disp) => format!("loop {}", format_jump(disp)),
            Instruction::LoopWhileEqual(disp) => format!("loope {}", format_jump(disp)),
            Instruction::LoopWhileNotEqual(disp) => format!("loopne {}", format_jump(disp)),
            Instruction::JumpOnCxZero(disp) => format!("jcxz {}", format_jump(disp)),
        }
    }
}
