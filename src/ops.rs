use std::fmt::Display;

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
