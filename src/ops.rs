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
pub struct RegisterAccess {
    pub reg: Register,
    pub width: OpWidth,
    pub offset: u8,
}

impl RegisterAccess {
    pub fn new(reg: Register, width: OpWidth, offset: u8) -> RegisterAccess {
        RegisterAccess { reg , width, offset }
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

            RegOrMem::Mem(EffectiveAddress { base, displacement })
                if matches!(base, &EffectiveAddressBase::Direct) =>
            {
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
#[derive(Debug, Clone)]
pub enum OpCode {
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
    ArithmeticFromToRegMem {
        op: ArithmeticOp,
        dir: Direction,
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

impl OpCode {
    pub fn encode(&self, current_i: usize) -> String {
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
            OpCode::JumpOnEqual(disp) => format!("je {}", to_label(disp, current_i)),
            OpCode::JumpOnLess(disp) => format!("jl {}", to_label(disp, current_i)),
            OpCode::JumpOnNotGreater(disp) => format!("jle {}", to_label(disp, current_i)),
            OpCode::JumpOnBelow(disp) => format!("jb {}", to_label(disp, current_i)),
            OpCode::JumpOnNotAbove(disp) => format!("jbe {}", to_label(disp, current_i)),
            OpCode::JumpOnParity(disp) => format!("jp {}", to_label(disp, current_i)),
            OpCode::JumpOnOverflow(disp) => format!("jo {}", to_label(disp, current_i)),
            OpCode::JumpOnSign(disp) => format!("js {}", to_label(disp, current_i)),
            OpCode::JumpOnNotEqual(disp) => format!("jne {}", to_label(disp, current_i)),
            OpCode::JumpOnNotLess(disp) => format!("jnl {}", to_label(disp, current_i)),
            OpCode::JumpOnGreater(disp) => format!("jg {}", to_label(disp, current_i)),
            OpCode::JumpOnNotBelow(disp) => format!("jnb {}", to_label(disp, current_i)),
            OpCode::JumpOnAbove(disp) => format!("jnbe {}", to_label(disp, current_i)),
            OpCode::JumpOnNoParity(disp) => format!("jnp {}", to_label(disp, current_i)),
            OpCode::JumpOnNoOverflow(disp) => format!("jno {}", to_label(disp, current_i)),
            OpCode::JumpOnNotSign(disp) => format!("jns {}", to_label(disp, current_i)),
            OpCode::Loop(disp) => format!("loop {}", to_label(disp, current_i)),
            OpCode::LoopWhileEqual(disp) => format!("loope {}", to_label(disp, current_i)),
            OpCode::LoopWhileNotEqual(disp) => format!("loopne {}", to_label(disp, current_i)),
            OpCode::JumpOnCxZero(disp) => format!("jcxz {}", to_label(disp, current_i)),
        }
    }
}

fn to_label(disp: i8, current_i: usize) -> String {
    let target = if disp < 0 {
        let disp = -disp as usize;
        current_i.checked_sub(disp).unwrap()
    } else {
        let disp = disp as usize;
        current_i + disp
    };
    format!("label_{target}")
}
