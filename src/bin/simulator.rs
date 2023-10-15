use std::{env, error::Error, fmt::Debug, fs, path::Path};
use std::fs::File;

use sim8086::{
    decoder::Decoder,
    flag_registers::Flags,
    ops::{ArithmeticOp, Instruction, OpWidth, Register, RegisterAccess, SegmentRegister, RegOrMem},
};
use sim8086::memory::Memory;
use sim8086::ops::{Direction, EffectiveAddress, EffectiveAddressBase};

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
                ip: 0,
                flags: Flags::empty(),
            },
        }
    }
}

#[derive(Debug)]
struct Registers {
    regs: [i16; 8],     //layout: AX, BX, CX, DX, SP, BP, SI, DI
    seg_regs: [i16; 4], //layout: ES, CS, SS, DS
    ip: usize,
    flags: Flags,
}

fn print_reg(name: &str, old: i16, new: i16) {
    print!("{}:0x{:x}->0x{:x}", name, old, new);
}

impl Registers {

    fn ax(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::A, width: OpWidth::Word, offset: 0 })
    }

    fn bx(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::B, width: OpWidth::Word, offset: 0 })
    }

    fn cx(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::C, width: OpWidth::Word, offset: 0 })
    }

    fn set_cx(&mut self, value: i16) {
        self.write_reg(value, RegisterAccess { reg: Register::C, width: OpWidth::Word, offset: 0 });
    }

    fn dx(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::D, width: OpWidth::Word, offset: 0 })
    }

    fn bp(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::Bp, width: OpWidth::Word, offset: 0 })
    }

    fn si(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::Si, width: OpWidth::Word, offset: 0 })
    }

    fn sp(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::Sp, width: OpWidth::Word, offset: 0 })
    }

    fn di(&self) -> i16 {
        self.read_reg(RegisterAccess { reg: Register::Di, width: OpWidth::Word, offset: 0 })
    }

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
                    B => self.regs[1],
                    C => self.regs[2],
                    D => self.regs[3],
                    _ => panic!("impossible"),
                },
                OpWidth::Byte => {
                    let word = match reg.reg {
                        A => self.regs[0],
                        B => self.regs[1],
                        C => self.regs[2],
                        D => self.regs[3],
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
            Sp => {
                print_reg("sp", self.regs[4], value);
                self.regs[4] = value
            }
            Bp => {
                print_reg("bp", self.regs[5], value);
                self.regs[5] = value
            }
            Si => {
                print_reg("si", self.regs[6], value);
                self.regs[6] = value
            }
            Di => {
                print_reg("di", self.regs[7], value);
                self.regs[7] = value
            }
            _ => match reg.width {
                OpWidth::Word => match reg.reg {
                    A => {
                        print_reg("ax", self.regs[0], value);
                        self.regs[0] = value
                    }
                    B => {
                        print_reg("bx", self.regs[1], value);
                        self.regs[1] = value
                    }
                    C => {
                        print_reg("cx", self.regs[2], value);
                        self.regs[2] = value
                    }
                    D => {
                        print_reg("dx", self.regs[3], value);
                        self.regs[3] = value
                    }
                    _ => panic!("impossible"),
                },
                OpWidth::Byte => {
                    assert!(value <= i8::MAX as i16);
                    assert!(value >= i8::MIN as i16);

                    let original: i16 = match reg.reg {
                        A => self.regs[0],
                        B => self.regs[1],
                        C => self.regs[2],
                        D => self.regs[3],
                        _ => panic!("impossible"),
                    };
                    let new = if reg.offset != 0 {
                        (original & 0x00FF) | (value << 8)
                    } else {
                        (original & -256/* 0xFF00 */) | value
                    };
                    match reg.reg {
                        A => {
                            print_reg("ax", self.regs[0], new);
                            self.regs[0] = new
                        }
                        B => {
                            print_reg("bx", self.regs[1], new);
                            self.regs[1] = new
                        }
                        C => {
                            print_reg("cx", self.regs[2], new);
                            self.regs[2] = new
                        }
                        D => {
                            print_reg("dx", self.regs[3], new);
                            self.regs[3] = new
                        }
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
            Es => {
                print_reg("es", self.seg_regs[0], value);
                self.seg_regs[0] = value
            }
            Cs => {
                print_reg("cs", self.seg_regs[1], value);
                self.seg_regs[1] = value
            }
            Ss => {
                print_reg("ss", self.seg_regs[2], value);
                self.seg_regs[2] = value
            }
            Ds => {
                print_reg("ds", self.seg_regs[3], value);
                self.seg_regs[3] = value
            }
        }
    }
}

fn simulate<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let decoder = Decoder::new();

    let path = path.as_ref();
    let bytes = fs::read(path)?;
    let program_length = bytes.len();

    let mut state = CpuState::new();
    let mut memory = Memory::new();
    memory.copy_from_slice(&bytes, 0);

    loop {
        let ip_before = state.registers.ip;
        let instruction = {
            let mut iter = memory.iter(state.registers.ip, program_length).enumerate().peekable();
            let instruction = decoder.decode_next(&mut iter.by_ref().map(|(_i, byte)| byte));
            let instruction_len = iter.peek().map(|(i, _u)| *i).unwrap_or(program_length - ip_before);
            state.registers.ip += instruction_len;

            if instruction.is_none() {
                break;
            }

            instruction.unwrap()
        };

        print!("{:<20} ; ", instruction.encode(|disp| format!("{disp}")));
        simulate_instruction(&mut state, &mut memory, instruction);
        println!();
    }
    println!();
    println!("Final registers:");
    print_register("ax", state.registers.regs[0]);
    print_register("bx", state.registers.regs[1]);
    print_register("cx", state.registers.regs[2]);
    print_register("dx", state.registers.regs[3]);
    print_register("sp", state.registers.regs[4]);
    print_register("bp", state.registers.regs[5]);
    print_register("si", state.registers.regs[6]);
    print_register("di", state.registers.regs[7]);
    print_register("es", state.registers.seg_regs[0]);
    print_register("cs", state.registers.seg_regs[1]);
    print_register("ss", state.registers.seg_regs[2]);
    print_register("ds", state.registers.seg_regs[3]);

    println!("    ip: 0x{:04x} ({0})", state.registers.ip);
    if !state.registers.flags.is_empty() {
        println!(" flags: {}", state.registers.flags);
    }

    let with_extension = path.with_extension("data");
    let x = with_extension.file_name().unwrap();

    memory.dump(&mut File::create(x).unwrap());

    Ok(())
}

fn print_register(name: &str, value: i16) {
    if value == 0 { return; }
    let value = value as u16;
    println!("    {name}: 0x{value:04x} ({value})");
}

fn calculate_address(ea: EffectiveAddress, state: &CpuState) -> i16 {
    if matches!(ea.base, EffectiveAddressBase::Direct) {
        return ea.displacement;
    }

    let base = match ea.base {
        EffectiveAddressBase::Direct => { panic!() }
        EffectiveAddressBase::BxPlusSi => {state.registers.bx() + state.registers.si()}
        EffectiveAddressBase::BxPlusDi => {state.registers.bx() + state.registers.di()}
        EffectiveAddressBase::BpPlusSi => {state.registers.bp() + state.registers.si()}
        EffectiveAddressBase::BpPlusDi => {state.registers.bp() + state.registers.di()}
        EffectiveAddressBase::Si => {state.registers.si()}
        EffectiveAddressBase::Di => {state.registers.di()}
        EffectiveAddressBase::Bp => {state.registers.bp()}
        EffectiveAddressBase::Bx => {state.registers.bx()}
    };

    base + ea.displacement
}

fn read_mem(mem: &Memory, state: &CpuState, effective_address: EffectiveAddress, width: OpWidth) -> i16 {
    let address = calculate_address(effective_address, state);
    if address < 0 { panic!(); }

    match width {
        OpWidth::Byte => {*mem.get(address as usize).unwrap() as i16}
        OpWidth::Word => {
            let lo = *mem.get(address as usize).unwrap();
            let hi = *mem.get((address as usize) + 1).unwrap();
            i16::from_le_bytes([lo, hi])
        }
    }
}

fn write_mem(value: i16, mem: &mut Memory, state: &CpuState, effective_address: EffectiveAddress, width: OpWidth) {
    let address = calculate_address(effective_address, state);
    if address < 0 { panic!(); }
    let address = address as usize;

    match width {
        OpWidth::Byte => {
            let value = value as u8;
            mem.set(value , address);
        }
        OpWidth::Word => {
            let le_bytes = value.to_le_bytes();
            mem.set(le_bytes[0], address);
            mem.set(le_bytes[1], address + 1);
        }
    }
}

fn simulate_instruction(state: &mut CpuState, memory: &mut Memory, instruction: Instruction) {
    match instruction {
        Instruction::ImmediateMovReg { reg, data } => {
            state.registers.write_reg(data, reg);
        }
        Instruction::ImmediateMovRegMem {
            width,
            reg_or_mem,
            data,
        } => {
            match reg_or_mem {
                RegOrMem::Mem(ea) => {
                    write_mem(data, memory, state, ea, width)
                },
                RegOrMem::Reg(access) => {
                    state.registers.write_reg(data, access);
                }
            }
        }
        Instruction::MovToFromRegMem { dir, reg, reg_or_mem } => match reg_or_mem {
            RegOrMem::Mem(effective_address) => match dir {
                Direction::FromRegister => {
                    let v = state.registers.read_reg(reg);
                    write_mem(v, memory, state, effective_address, reg.width);
                },
                Direction::ToRegister => {
                    let m = read_mem(memory, state, effective_address, reg.width);
                    state.registers.write_reg(m, reg);
                }
            },
            RegOrMem::Reg(reg_access) => match dir {
                Direction::FromRegister => {
                    let v = state.registers.read_reg(reg);
                    state.registers.write_reg(v, reg_access)
                }
                Direction::ToRegister => {
                    let v = state.registers.read_reg(reg_access);
                    state.registers.write_reg(v, reg)
                }
            },
        },
        Instruction::SegmentRegisterMove { dir, seg_reg, reg_or_mem } => match reg_or_mem {
            RegOrMem::Mem(_) => todo!(),
            RegOrMem::Reg(reg_access) => match dir {
                Direction::FromRegister => {
                    let value = state.registers.read_seg_reg(seg_reg);
                    state.registers.write_reg(value, reg_access);
                }
                Direction::ToRegister => {
                    let value = state.registers.read_reg(reg_access);
                    state.registers.write_seg_reg(seg_reg, value);
                }
            },
        },
        Instruction::ArithmeticFromToRegMem { op, dir, width, reg, reg_or_mem } => match reg_or_mem {
            RegOrMem::Mem(ea) => match dir {
                Direction::ToRegister => {
                    let one = state.registers.read_reg(reg);
                    let two = read_mem(memory, state, ea, width);
                    let (result, flags) = evaluate_op(op, one, two);
                    if store_result(op) {
                        state.registers.write_reg(result, reg);
                    }
                    update_flags(state, (result, flags), Flags::arithmetic_flags());
                }
                Direction::FromRegister => {
                    let one = read_mem(memory, state, ea, width);
                    let two = state.registers.read_reg(reg);
                    let (result, flags) = evaluate_op(op, one, two);
                    if store_result(op) {
                        write_mem(result, memory, state, ea, width);
                    }
                    update_flags(state, (result, flags), Flags::arithmetic_flags());
                }
            },
            RegOrMem::Reg(reg_access) => match dir {
                Direction::FromRegister => {
                    let one = state.registers.read_reg(reg_access);
                    let two = state.registers.read_reg(reg);
                    let (result, flags) = evaluate_op(op, one, two);
                    if store_result(op) {
                        state.registers.write_reg(result, reg_access);
                    }
                    update_flags(state, (result, flags), Flags::arithmetic_flags());
                }
                Direction::ToRegister => {
                    let one = state.registers.read_reg(reg);
                    let two = state.registers.read_reg(reg_access);
                    let (result, flags) = evaluate_op(op, one, two);
                    if store_result(op) {
                        state.registers.write_reg(result, reg);
                    }
                    update_flags(state, (result, flags), Flags::arithmetic_flags());
                }
            },
        },
        Instruction::ArithmeticImmediateToRegMem { op, width, data, reg_or_mem } => match reg_or_mem {
            RegOrMem::Mem(ea) => {
                let one = read_mem(memory, state, ea, width);
                let two = data;
                let (result, flags) = evaluate_op(op, one, two);
                if store_result(op) {
                    write_mem(result, memory, state, ea, width);
                }
                update_flags(state, (result, flags), Flags::arithmetic_flags());
            },
            RegOrMem::Reg(reg_access) => {
                let one = state.registers.read_reg(reg_access);
                let two = data;
                let (result, flags) = evaluate_op(op, one, two);
                if store_result(op) {
                    state.registers.write_reg(result, reg_access);
                }
                update_flags(state, (result, flags), Flags::arithmetic_flags());
            }
        },
        Instruction::JumpOnEqual(offset) => {
            if state.registers.flags.contains(Flags::Zero) {
                state.registers.ip = state.registers.ip.overflowing_add_signed(offset as isize).0;
            }
        },
        Instruction::JumpOnNotEqual(offset) => {
            if !state.registers.flags.contains(Flags::Zero) {
                state.registers.ip = state.registers.ip.overflowing_add_signed(offset as isize).0;
            }
        },
        Instruction::JumpOnSign(offset) => {
            if state.registers.flags.contains(Flags::Sign) {
                state.registers.ip = state.registers.ip.overflowing_add_signed(offset as isize).0;
            }
        },
        Instruction::JumpOnNotSign(offset) => {
            if !state.registers.flags.contains(Flags::Sign) {
                state.registers.ip = state.registers.ip.overflowing_add_signed(offset as isize).0;
            }
        },
        Instruction::Loop(offset) => {
            let cx = state.registers.cx() - 1;
            state.registers.set_cx(cx);
            if cx != 0 {
                let (ip, overflow) = state.registers.ip.overflowing_add_signed(offset as isize);
                if overflow { panic!(); }
                state.registers.ip = ip;
            }
        },
        Instruction::LoopWhileEqual(offset) => {
            let cx = state.registers.cx() - 1;
            state.registers.set_cx(cx);
            if cx != 0 && state.registers.flags.contains(Flags::Zero) {
                let (ip, overflow) = state.registers.ip.overflowing_add_signed(offset as isize);
                if overflow { panic!(); }
                state.registers.ip = ip;
            };
        },
        Instruction::LoopWhileNotEqual(offset) => {
            let cx = state.registers.cx() - 1;
            state.registers.set_cx(cx);
            if cx != 0 && !state.registers.flags.contains(Flags::Zero) {
                let (ip, overflow) = state.registers.ip.overflowing_add_signed(offset as isize);
                if overflow { panic!(); }
                state.registers.ip = ip;
            };
        },
        _ => todo!(),
    }
}

fn evaluate_op(op: ArithmeticOp, one: i16, two: i16) -> (i16, Flags) {
    match op {
        ArithmeticOp::Add => {
            let (result, overflow) = one.overflowing_add(two);

            let mut flags = Flags::empty();
            if overflow { flags |= Flags::Overflow; }
            if (result as u16) < (one as u16) { flags |= Flags::Carry }

            let o = (one as u16) & 0xf;
            let t = (two as u16) & 0xf;
            let aux_carry = (o + t) >= 16;
            if aux_carry {flags |= Flags::AuxiliaryCarry}

            (result, flags)
        }
        ArithmeticOp::Sub | ArithmeticOp::Cmp => {
            let (result, overflow) = one.overflowing_sub(two);

            let mut flags = Flags::empty();
            if overflow { flags |= Flags::Overflow }
            if (result as u16) > (one as u16) { flags |= Flags::Carry }

            let o = (one as u16) & 0xf;
            let t = (two as u16) & 0xf;
            let (_, aux_carry) = o.overflowing_sub(t);
            if aux_carry {flags |= Flags::AuxiliaryCarry}

            (result, flags)
        }
        _ => todo!(),
    }
}

fn update_flags(state: &mut CpuState, result: (i16, Flags), flags: Flags) {
    print!(" flags:{}", state.registers.flags);

    let (result, op_flags) = result;
    if flags.contains(Flags::Zero) {
        state.registers.flags.set(Flags::Zero, result == 0);
    }
    if flags.contains(Flags::Sign) {
        state.registers.flags.set(Flags::Sign, result < 0);
    }
    if flags.contains(Flags::Parity) {
        /* From the Intel manual:
                PF (parity flag): If the low-order eight bits of
                an arithmetic or logical result contain an
                even number of 1-bits, then the parity flag is
                set; otherwise it is cleared. PF is provided for
                8080/8085 compatibility; it also can be used
                to check ASCII characters for correct parity. */
        state.registers.flags.set(Flags::Parity, (result & 0xff).count_ones() % 2 == 0);
    }
    if flags.contains(Flags::Carry) {
        state.registers.flags.set(Flags::Carry, op_flags.contains(Flags::Carry));
    }
    if flags.contains(Flags::Overflow) {
        state.registers.flags.set(Flags::Overflow, op_flags.contains(Flags::Overflow));
    }
    if flags.contains(Flags::AuxiliaryCarry) {
        state.registers.flags.set(Flags::AuxiliaryCarry, op_flags.contains(Flags::AuxiliaryCarry));
    }
    print!("->{}", state.registers.flags);
}

fn store_result(op: ArithmeticOp) -> bool {
    match op {
        ArithmeticOp::Cmp => false,
        _ => true,
    }
}

#[cfg(test)]
mod test {
    use sim8086::flag_registers::Flags;
    use sim8086::ops::ArithmeticOp;
    use crate::evaluate_op;

    #[test]
    fn evaluate_op_add_overflow() {
        let (result, flags) = evaluate_op(ArithmeticOp::Add, i16::MAX, 1);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::Overflow));
    }

    #[test]
    fn evaluate_op_add_carry() {
        let (result, flags) = evaluate_op(ArithmeticOp::Add, -1, 1);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::Carry));
    }

    #[test]
    fn evaluate_op_add_aux_carry() {
        let (result, flags) = evaluate_op(ArithmeticOp::Add, 10, 10);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::AuxiliaryCarry));
    }

    #[test]
    fn evaluate_op_sub_overflow() {
        let (result, flags) = evaluate_op(ArithmeticOp::Sub, i16::MIN, 1);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::Overflow));
    }

    #[test]
    fn evaluate_op_sub_carry() {
        let (result, flags) = evaluate_op(ArithmeticOp::Sub, 0, 1);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::Carry));
    }

    #[test]
    fn evaluate_op_sub_aux_carry() {
        let (result, flags) = evaluate_op(ArithmeticOp::Sub, 20, 10);
        let s = format!("{}", flags);
        print!("{}", s);

        assert!(flags.contains(Flags::AuxiliaryCarry));
    }
}