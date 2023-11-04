use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::exit;

use rand::prelude::*;

use sim8086::haversine;

const USAGE: &'static str = "Usage: haversine_generator uniform|clustered seed point_count";
const CLUSTER_COUNT: usize = 64;

#[derive(Debug)]
enum Mode {
    Uniform,
    Clustered,
}

fn usage() {
    println!("{}", USAGE);
    exit(1);
}

fn main() {
    let config = parse_args();

    let json = format!("haversine_input_{}.json", config.count);
    let reference_answers = format!("haversine_results_{}.data", config.count);

    let mut json = BufWriter::new(File::create(json).unwrap());
    let mut reference_answers = BufWriter::new(File::create(reference_answers).unwrap());

    write!(&mut json, "{{\"pairs\": [\n");
    let mut rng = StdRng::seed_from_u64(config.seed);

    let mut points_left = match config.mode {
        Mode::Uniform => { usize::MAX }
        Mode::Clustered => { 0 }
    };
    let mut xc: f64 = 0.0;
    let mut yc: f64 = 0.0;
    let mut x_spread = 180.0;
    let mut y_spread = 90.0;

    let coeff = 1.0 / (config.count as f64);
    let mut sum = 0.0;
    for (i, _) in (0..config.count).enumerate() {
        if points_left == 0 {
            points_left = 1 + config.count / CLUSTER_COUNT;
            x_spread = rng.gen_range(0.0..180.0);
            y_spread = rng.gen_range(0.0..90.0);
            xc = rng.gen_range(-180.0 + x_spread..180.0 - x_spread);
            yc = rng.gen_range(-90.0 + y_spread..90.0 - y_spread);
        }
        points_left -= 1;

        let x0: f64 = xc + rng.gen_range(-x_spread..x_spread);
        let y0: f64 = yc + rng.gen_range(-y_spread..y_spread);
        let x1: f64 = xc + rng.gen_range(-x_spread..x_spread);
        let y1: f64 = yc + rng.gen_range(-y_spread..y_spread);

        assert!(x0 >= -180.0);
        assert!(x0 <= 180.0);
        assert!(x1 >= -180.0);
        assert!(x1 <= 180.0);
        assert!(y0 >= -90.0);
        assert!(y0 <= 90.0);
        assert!(y1 >= -90.0);
        assert!(y1 <= 90.0);

        write!(&mut json, "  {{\"x0\": {x0}, \"y0\": {y0}, \"x1\": {x1}, \"y1\": {y1} }}");
        if i < config.count - 1 { write!(&mut json, ","); }
        write!(&mut json, "\n");

        let reference = haversine::reference_haversine(x0, y0, x1, y1, 6372.8f64);
        sum += reference * coeff;

        reference_answers.write_all(reference.to_le_bytes().as_slice()).unwrap();
    }
    write!(&mut json, "]}}");

    println!("Sum: {sum}");
    reference_answers.write_all(sum.to_le_bytes().as_slice()).unwrap();
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();

    if (args.len() < 4) {
        usage();
        unreachable!("usage() should terminate the program")
    }
    let mode = args[1].as_str();
    let mode = match mode {
        "uniform" => Mode::Uniform,
        "clustered" => Mode::Clustered,
        _ => {
            usage();
            unreachable!("usage() should terminate the program")
        }
    };

    println!("Mode: {mode:?}");

    let seed = args[2].as_str();
    let seed = match seed.parse::<u64>() {
        Ok(seed) => { seed }
        Err(err) => {
            println!("'{seed}' is not a valid seed. Seed must be a positive integer");
            usage();
            unreachable!()
        }
    };

    let count = args[3].as_str();
    let count = match count.parse::<usize>() {
        Ok(count) => { count }
        Err(_) => {
            println!("'{count}' is not a valid count. Count should be a positive integer");
            unreachable!();
        }
    };

    Config {
        mode,
        seed,
        count,
    }
}

struct Config {
    mode: Mode,
    seed: u64,
    count: usize,
}