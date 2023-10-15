use std::io::{self, Write};

pub struct Memory {
    data: Box<[u8]>,
}

impl Memory {
    pub fn new() -> Box<Memory> {
        Box::new(Memory {
            data: vec![0u8; 1024 * 1024].into_boxed_slice(),
        })
    }

    pub fn copy_from_slice(&mut self, data: &[u8], offset: usize) {
        self.data[offset..offset + data.len()].copy_from_slice(data);
    }

    pub fn iter(&self, offset: usize, limit: usize) -> impl Iterator<Item = &u8> {
      self.data[offset..limit].iter()
    }

    pub fn get(&self, offset: usize) -> Option<&u8> {
        if offset < self.data.len() {
            Some(&self.data[offset])
        } else {
            None
        }
    }

    pub fn dump<W: Write>(&self, write: &mut W) -> io::Result<()> {
        let mut written = 0;
        while written < self.data.len() {
            match write.write(&self.data) {
                Ok(n) => written += n,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}
