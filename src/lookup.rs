use crate::decode::*;
use std::collections::HashMap;

pub struct OpDecoderLookup {
    map: HashMap<u8, Box<dyn OpCodeDecoder>>,
}

impl OpDecoderLookup {
    pub fn new() -> OpDecoderLookup {
        OpDecoderLookup { map: HashMap::new() }
    }

    pub fn insert<D: OpCodeDecoder + Clone + 'static>(&mut self, pattern: &str, decoder: D) {
        let pattern = pattern.strip_prefix("0b").unwrap_or(pattern);
        let pattern = pattern.replace('_', "");

        let mut v = vec![];
        Self::expand(&pattern, &mut v);

        for b in v {
            self.map.insert(b, Box::new(decoder.clone()));
        }
    }

    pub fn get(&self, opcode: &u8) -> Option<&Box<dyn OpCodeDecoder>> {
        self.map.get(opcode)
    }

    fn expand(i: &str, v: &mut Vec<u8>) {
        if i.chars().all(|c| c == '1' || c == '0') {
            let p = u8::from_str_radix(i, 2).expect(&format!("could not parse {i}"));
            v.push(p);
        } else {
            Self::expand(&i.replacen(|c| !char::is_numeric(c), "0", 1), v);
            Self::expand(&i.replacen(|c| !char::is_numeric(c), "1", 1), v);
        }
    }
}
