use std::path::Path;
use std::{env, fs};

use sim8086::decoder::Decoder;

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

    let bytes = fs::read(path)?;
    let mut iter = bytes.iter().enumerate().peekable();
    println!("bits 16");

    loop {
        let position_before = match iter.peek() {
            Some((i, _byte)) => *i,
            None => bytes.len(),
        };

        let code = decoder.decode_next(&mut iter.by_ref().map(|(_i, byte)| byte));
        if code.is_none() {
            break;
        }

        let code = code.unwrap();

        let position_after = match iter.peek() {
            Some((i, _byte)) => *i,
            None => bytes.len(),
        };

        println!("label_{position_before}:");
        println!("{}", code.encode(position_after));
    }
    Ok(())
}
