use std::fs;

pub const BLOCK_SIZE: usize = 4096;
pub const TOTAL_BLOCKS: usize = 64;

#[derive(Debug)]
pub struct Hardware {
    pub data: Vec<u8>,
}

impl Hardware {
    pub fn new() -> Self {
        Self {
            data: vec![0; BLOCK_SIZE * TOTAL_BLOCKS],
        }
    }

    pub fn load(path: &str) -> Self {
        if fs::metadata(path).is_err() {
            fs::write(path, &vec![0; BLOCK_SIZE * TOTAL_BLOCKS]).unwrap();
        }

        let data = fs::read(path).unwrap();

        if data.len() != BLOCK_SIZE * TOTAL_BLOCKS {
            panic!("Invalid file size");
        }

        Self { data }
    }

    pub fn save(&self, path: &str) {
        fs::write(path, &self.data).unwrap();
    }
}
