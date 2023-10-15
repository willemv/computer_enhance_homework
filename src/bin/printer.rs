use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::path::Path;
use std::{env, fs};

use sim8086::decoder::Decoder;
use sim8086::memory::Memory;
use sim8086::ops::Instruction;

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
    let decoder = Decoder::new();
    let mut memory = Memory::new();

    let bytes = fs::read(path)?;
    memory.copy_from_slice(&bytes, 0);
    memory.dump(&mut File::create("scratch/dump.data")?)?;

    let mut iter = memory.iter(0, bytes.len()).enumerate().peekable();
    println!("bits 16");

    let mut decoded_instructions: Vec<(usize, usize, Instruction)> = vec![];
    loop {
        let position_before = match iter.peek() {
            Some((i, _byte)) => *i,
            None => bytes.len(),
        };

        let instruction = decoder.decode_next(&mut iter.by_ref().map(|(_i, byte)| byte));
        if instruction.is_none() {
            break;
        }

        let instruction = instruction.unwrap();

        let position_after = match iter.peek() {
            Some((i, _byte)) => *i,
            None => bytes.len(),
        };

        decoded_instructions.push((position_before, position_after, instruction));
    }

    //collect jump targets
    let mut jump_targets: BTreeSet<usize> = BTreeSet::new();
    for (_position_before, position_after, instruction) in decoded_instructions.iter() {
        if let Some(jump) = relative_jump(instruction) {
            let target = to_absolute(jump, *position_after);
            jump_targets.insert(target);
        }
    }

    let mut jump_table = HashMap::new();
    for (i, target) in jump_targets.iter().enumerate() {
        jump_table.insert(*target, format!("label_{}", i + 1));
    }
    let jump_table = jump_table;

    for (position_before, position_after, instruction) in decoded_instructions {
        if let Some(label) = jump_table.get(&position_before) {
            println!("{}:", label);
        }
        println!("{}", instruction.encode(|disp| { to_label(disp, position_after, &jump_table) }));
    }

    Ok(())
}

fn to_label(disp: i8, current_i: usize, jump_table: &HashMap<usize, String>) -> String {
    let target = if disp < 0 {
        let disp = -disp as usize;
        current_i.checked_sub(disp).unwrap()
    } else {
        let disp = disp as usize;
        current_i + disp
    };
    jump_table.get(&target).unwrap().clone()
}

fn to_absolute(disp: i8, current_i: usize) -> usize {
    if disp < 0 {
        let disp = -disp as usize;
        current_i.checked_sub(disp).unwrap()
    } else {
        let disp = disp as usize;
        current_i + disp
    }
}

fn relative_jump(instruction: &Instruction) -> Option<i8> {
    match *instruction {
        Instruction::JumpOnEqual(disp) => Some(disp),
        Instruction::JumpOnLess(disp) => Some(disp),
        Instruction::JumpOnNotGreater(disp) => Some(disp),
        Instruction::JumpOnBelow(disp) => Some(disp),
        Instruction::JumpOnNotAbove(disp) => Some(disp),
        Instruction::JumpOnParity(disp) => Some(disp),
        Instruction::JumpOnOverflow(disp) => Some(disp),
        Instruction::JumpOnSign(disp) => Some(disp),
        Instruction::JumpOnNotEqual(disp) => Some(disp),
        Instruction::JumpOnNotLess(disp) => Some(disp),
        Instruction::JumpOnGreater(disp) => Some(disp),
        Instruction::JumpOnNotBelow(disp) => Some(disp),
        Instruction::JumpOnAbove(disp) => Some(disp),
        Instruction::JumpOnNoParity(disp) => Some(disp),
        Instruction::JumpOnNoOverflow(disp) => Some(disp),
        Instruction::JumpOnNotSign(disp) => Some(disp),
        Instruction::Loop(disp) => Some(disp),
        Instruction::LoopWhileEqual(disp) => Some(disp),
        Instruction::LoopWhileNotEqual(disp) => Some(disp),
        Instruction::JumpOnCxZero(disp) => Some(disp),
        _ => None,
    }
}
