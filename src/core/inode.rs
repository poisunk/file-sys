const INODE_SIZE: usize = 64;

#[derive(Debug)]
pub struct Inode {
    pub name: String,
    pub size: u32,
    pub block_pos: Vec<u32>,
}

impl Inode {
    pub fn init(&mut self, name: &str) {
        self.name = name.to_string();
        self.size = 0;
        self.block_pos = Vec::new();
    }

    pub fn clean(&mut self) {
        self.name = String::new();
        self.size = 0;
        self.block_pos = Vec::new();
    }

    pub fn from_block_bytes(data: &[u8]) -> Vec<Inode> {
        let inodes = data
            .chunks_exact(INODE_SIZE)
            .map(|chunk| {
                let mut i = 0;
                let name_len = chunk[i];
                let name =
                    String::from_utf8(chunk[i + 1..i + 1 + name_len as usize].to_vec()).unwrap();

                i += 32;

                let size = u32::from_le_bytes(chunk[i..i + 4].try_into().unwrap());

                i += 4;

                let block_pos = chunk[i..i + 28]
                    .to_vec()
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();

                Inode {
                    name: name,
                    size: size,
                    block_pos: block_pos,
                }
            })
            .collect();
        inodes
    }

    pub fn to_le_bytes(&self) -> Vec<u8> {
        let mut i = 0;
        let mut raw_data = vec![0; 64];
        let name_len = self.name.len() as u8;
        if name_len > 32 {
            todo!("Name too long");
        }
        raw_data[i] = name_len;
        raw_data[i + 1..i + 1 + name_len as usize].copy_from_slice(self.name.as_bytes());
        i += 32;

        let size_data = self.size.to_le_bytes();
        raw_data[i..i + 4].copy_from_slice(&size_data);
        i += 4;

        let block_pos_data = self
            .block_pos
            .iter()
            .map(|x| x.to_le_bytes())
            .flatten()
            .collect::<Vec<u8>>();
        raw_data[i..i + block_pos_data.len()].copy_from_slice(&block_pos_data);

        raw_data
    }
}
