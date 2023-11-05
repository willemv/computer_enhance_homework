use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};

use sim8086::haversine;
use sim8086::json::JsonValue;

const USAGE: &'static str = r#"Usage: haversine_calculator path_to_json (path_to_reference)"#;

fn main() {
    match run() {
        Ok(_) => { println!("OK") }
        Err(e) => { println!("error: {e}") }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = read_args()?;

    let mut reference_reader = match args.reference {
        None => { None }
        Some(reference) => {
            Some(BufReader::new(File::open(reference)?))
        }
    };

    let json_path = args.json;
    let json = std::fs::read_to_string(json_path)?;
    let parsed = sim8086::json::parse_json_from_str(json.as_str()).ok_or("error decoding".to_owned())?;

    let mut sum = 0.0f64;
    let mut count = 0;
    let mut buf = [0u8; 8];
    match parsed {
        JsonValue::Object(root) => {
            let pairs = root.get("pairs").ok_or("pair".to_owned())?;
            match pairs {
                JsonValue::Array(pairs) => {
                    for pair in pairs {
                        match pair {
                            JsonValue::Object(pair) => {
                                let x0 = extract_number(pair, "x0")?;
                                let y0 = extract_number(pair, "y0")?;
                                let x1 = extract_number(pair, "x1")?;
                                let y1 = extract_number(pair, "y1")?;

                                let dist = haversine::reference_haversine(
                                    x0, y0, x1, y1, haversine::EARTH_RADIUS,
                                );

                                if (reference_reader.is_some()) {
                                    reference_reader.as_mut().unwrap().read_exact(&mut buf)?;
                                    let reference_dist = f64::from_le_bytes(buf);

                                    if (dist - reference_dist).abs() > f64::EPSILON {
                                        println!("Warning: bad dist!");
                                    }
                                }

                                sum += dist;
                                count += 1;
                            }
                            _ => return Err("each pair should be an object".into())
                        }
                    }
                }
                _ => return Err("pairs should be an array".into())
            }
        }
        _ => return Err("error".into())
    }

    let average = sum / count as f64;

    println!("Processed {count} pairs:");
    println!("  Average: {average}");
    if reference_reader.is_some() {
        reference_reader.as_mut().unwrap().read_exact(&mut buf);
        let reference_average = f64::from_le_bytes(buf);
        let diff = (average - reference_average).abs();
        println!("  Reference: {reference_average}");
        println!("  Diff: {diff}");
    }


    Ok(())
}

fn extract_number(object: &HashMap<String, JsonValue>, key: &str) -> Result<f64, Box<dyn Error>> {
    let value = object.get(key).ok_or(format!("value for {key} not found"))?;
    match value {
        JsonValue::Number(number) => { Ok(number.clone()) }
        _ => Err("not a number".into()),
    }
}

fn read_args() -> Result<Args, Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(USAGE.into());
    }

    let json = args[1].to_owned();

    let reference = if args.len() > 2 {
        Some(args[2].to_string())
    } else {
        None
    };

    Ok(Args {
        json,
        reference,
    })
}

struct Args {
    json: String,
    reference: Option<String>,
}