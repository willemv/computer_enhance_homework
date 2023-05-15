use std::{env, error::Error, fmt::Debug, fs, path::Path};

use sim8086::{
    decoder::Decoder,
    ops::{Instruction, OpWidth, Register, RegisterAccess, SegmentRegister},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        std::process::exit(1);
    }

    let assembly_file = &args[1];
    match simulate(assembly_file) {
        Ok(_) => println!("OK"),
        Err(_) => println!("Err"),
    };
}

#[derive(Debug)]
struct CpuState {
    registers: Registers,
    // memory: Memory
}

impl CpuState {
    fn new() -> CpuState {
        CpuState {
            registers: Registers {
                regs: [0i16; 8],
                seg_regs: [0i16; 4],
            },
        }
    }
}

#[derive(Debug)]
struct Registers {
    regs: [i16; 8],     //layout: AX, CX, DX, BX, SP, BP, SI, DI
    seg_regs: [i16; 4], //layout: ES, CS, SS, DS
}

impl Registers {
    fn read_reg(&self, reg: RegisterAccess) -> i16 {
        use Register::*;
        match reg.reg {
            Sp => self.regs[4],
            Bp => self.regs[5],
            Si => self.regs[6],
            Di => self.regs[7],
            _ => match reg.width {
                OpWidth::Word => match reg.reg {
                    A => self.regs[0],
                    C => self.regs[1],
                    D => self.regs[2],
                    B => self.regs[3],
                    _ => panic!("impossible"),
                },
                OpWidth::Byte => {
                    let word = match reg.reg {
                        A => self.regs[0],
                        C => self.regs[1],
                        D => self.regs[2],
                        B => self.regs[3],
                        _ => panic!("impossible"),
                    };
                    if reg.offset != 0 {
                        (word >> 8) & 0xFF
                    } else {
                        word & 0xFF
                    }
                }
            },
        }
    }   

    fn write_reg(&mut self, value: i16, reg: RegisterAccess) {
        use Register::*;
        match reg.reg {
            Sp => { print!("sp:0x{:0<4X}->0x{:0<4X}",self.regs[4], value); self.regs[4] = value},
            Bp => { print!("bp:0x{:0<4X}->0x{:0<4X}",self.regs[5], value); self.regs[5] = value},
            Si => { print!("si:0x{:0<4X}->0x{:0<4X}",self.regs[6], value); self.regs[6] = value},
            Di => { print!("di:0x{:0<4X}->0x{:0<4X}",self.regs[7], value); self.regs[7] = value},
            _ => match reg.width {
                OpWidth::Word => match reg.reg {
                    A => {print!("ax:0x{:0<4X}->0x{:0<4X}", self.regs[0], value) ; self.regs[0] = value},
                    C => {print!("cx:0x{:0<4X}->0x{:0<4X}", self.regs[1], value) ; self.regs[1] = value},
                    D => {print!("dx:0x{:0<4X}->0x{:0<4X}", self.regs[2], value) ; self.regs[2] = value},
                    B => {print!("bx:0x{:0<4X}->0x{:0<4X}", self.regs[3], value) ; self.regs[3] = value},
                    _ => panic!("impossible"),
                },
                OpWidth::Byte => {
                    assert!(value <= i8::MAX as i16);
                    assert!(value >= i8::MIN as i16);

                    let original: i16 = match reg.reg {
                        A => self.regs[0],
                        C => self.regs[1],
                        D => self.regs[2],
                        B => self.regs[3],
                        _ => panic!("impossible"),
                    };
                    let new = if reg.offset != 0 {
                        (original & 0x00FF) | (value << 8)
                    } else {
                        (original & -256/* 0xFF00 */) | value
                    };
                    match reg.reg {
                        A => {print!("ax:0x{:0<4X}->0x{:0<4X}", self.regs[0], new ); self.regs[0] = new},
                        C => {print!("cx:0x{:0<4X}->0x{:0<4X}", self.regs[1], new ); self.regs[1] = new},
                        D => {print!("dx:0x{:0<4X}->0x{:0<4X}", self.regs[2], new ); self.regs[2] = new},
                        B => {print!("bx:0x{:0<4X}->0x{:0<4X}", self.regs[3], new ); self.regs[3] = new},
                        _ => panic!("impossible"),
                    };
                }
            },
        }
    }

    fn read_seg_reg(&self, reg: SegmentRegister) -> i16 {
        use SegmentRegister::*;
        match reg {
            Es => self.seg_regs[0],
            Cs => self.seg_regs[1],
            Ss => self.seg_regs[2],
            Ds => self.seg_regs[3],
        }
    }

    fn write_seg_reg(&mut self, reg: SegmentRegister, value: i16) {
        use SegmentRegister::*;
        match reg {
            Es => {print!("es:0x{:0<4X}->0x{:0<4X}", self.seg_regs[0], value); self.seg_regs[0] = value},
            Cs => {print!("cs:0x{:0<4X}->0x{:0<4X}", self.seg_regs[1], value); self.seg_regs[1] = value},
            Ss => {print!("ss:0x{:0<4X}->0x{:0<4X}", self.seg_regs[2], value); self.seg_regs[2] = value},
            Ds => {print!("ds:0x{:0<4X}->0x{:0<4X}", self.seg_regs[3], value); self.seg_regs[3] = value},
        }
    }
}

fn simulate<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let decoder = Decoder::new();

    let bytes = fs::read(path)?;
    let mut iter = bytes.iter();

    let mut state = CpuState::new();

    loop {
        let instruction = decoder.decode_next(&mut iter);
        if instruction.is_none() {
            break;
        }

        let instruction = instruction.unwrap();

        print!("{:<20} ; ", instruction.encode(|disp| format!("{disp}")));
        simulate_instruction(&mut state, instruction);

        println!("");
    }
    println!();
    println!("Final registers:");
    println!("    ax: 0x{:0<4X} ({0})", state.registers.regs[0]);
    println!("    bx: 0x{:0<4X} ({0})", state.registers.regs[3]);
    println!("    cx: 0x{:0<4X} ({0})", state.registers.regs[1]);
    println!("    dx: 0x{:0<4X} ({0})", state.registers.regs[2]);
    println!("    sp: 0x{:0<4X} ({0})", state.registers.regs[4]);
    println!("    bp: 0x{:0<4X} ({0})", state.registers.regs[5]);
    println!("    si: 0x{:0<4X} ({0})", state.registers.regs[6]);
    println!("    di: 0x{:0<4X} ({0})", state.registers.regs[7]);
    println!("    es: 0x{:0<4X} ({0})", state.registers.seg_regs[0]);
    println!("    cs: 0x{:0<4X} ({0})", state.registers.seg_regs[1]);
    println!("    ss: 0x{:0<4X} ({0})", state.registers.seg_regs[2]);
    println!("    ds: 0x{:0<4X} ({0})", state.registers.seg_regs[3]);

    Ok(())
}

fn simulate_instruction(state: &mut CpuState, instruction: Instruction) {
    match instruction {
        Instruction::ImmediateMovReg { reg, data } => {
            state.registers.write_reg(data, reg);
        }
        Instruction::ImmediateMovRegMem { width: _, reg_or_mem, data } => {
            match reg_or_mem {
                sim8086::ops::RegOrMem::Mem(ref _ea) => todo!(),
                sim8086::ops::RegOrMem::Reg(access) => {
                    state.registers.write_reg(data, access);
                }
            }
            todo!()
        }
        Instruction::MovToFromRegMem { dir, reg, reg_or_mem } => match reg_or_mem {
            sim8086::ops::RegOrMem::Mem(_) => todo!(),
            sim8086::ops::RegOrMem::Reg(reg_access) => match dir {
                sim8086::ops::Direction::FromRegister => {
                    let v = state.registers.read_reg(reg);
                    state.registers.write_reg(v, reg_access)
                }
                sim8086::ops::Direction::ToRegister => {
                    let v = state.registers.read_reg(reg_access);
                    state.registers.write_reg(v, reg)
                }
            },
        },
        Instruction::SegmentRegisterMove { dir, seg_reg, reg_or_mem } => match reg_or_mem {
            sim8086::ops::RegOrMem::Mem(_) => todo!(),
            sim8086::ops::RegOrMem::Reg(reg_access) => match dir {
                sim8086::ops::Direction::FromRegister => {
                    let value = state.registers.read_seg_reg(seg_reg);
                    state.registers.write_reg(value, reg_access);
                }
                sim8086::ops::Direction::ToRegister => {
                    let value = state.registers.read_reg(reg_access);
                    state.registers.write_seg_reg(seg_reg, value);
                }
            },
        },
        _ => todo!(),
    }
}
