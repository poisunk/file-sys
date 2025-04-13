
#[derive(Debug)]
pub struct File {
    pub name: String,
    pub inode_index: usize,
    pub size: u32,
    pub content: String,
}

impl File {
    pub fn new(name: &str, inode_index: usize) -> Self {
        Self {
            name: name.to_string(),
            inode_index: inode_index,
            size: 0,
            content: "".to_string(),
        }
    }

    pub fn from_block_bytes(name: &str, inode_index: usize, data: &[u8]) -> Self {
        let mut file = Self::new(name, inode_index);

        let data: Vec<u8> = data
            .iter()
            .filter(|x| **x != 0)
            .map(|x| *x)
            .collect();

        let content = String::from_utf8(data.to_vec()).unwrap();

        let size = data.len() as u32;

        file.size = size;
        file.content = content;

        file
    }

    pub fn show(&self) {
        println!("{}", self.content);
    }
}
