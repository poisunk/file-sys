#[derive(Debug)]
pub struct Dir {
    pub name: String,
    pub inode_index: usize,
    pub items: Vec<DirItem>,
}

impl Dir {
    pub fn new(name: &str, inode_index: usize) -> Self {
        Self {
            name: name.to_string(),
            inode_index: inode_index,
            items: Vec::new(),
        }
    }

    pub fn from_block_bytes(name: &str, inode_index: usize, data: &[u8]) -> Self {
        let mut dir = Self::new(name, inode_index);
        let mut i = 0;

        while i < data.len() {
            let inode_pos = u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
            if inode_pos == 0xDEADBEAF {
                break;
            }
            i += 4;
            
            let name_len = u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
            let name = String::from_utf8(data[i + 4..i + 4 + name_len as usize].to_vec()).unwrap();

            i += 4 + name_len as usize;

            let typ_len = u32::from_le_bytes(data[i ..i + 4 ].try_into().unwrap());
            let typ = String::from_utf8(data[i + 4..i + 4 + typ_len as usize].to_vec()).unwrap();
            
            i += 4 + typ_len as usize;

            let size = u32::from_le_bytes(data[i..i + 4].try_into().unwrap());
            i += 4;

            dir.items.push(DirItem {
                inode_pos: inode_pos,
                name: name,
                typ: typ,
                size: size,
            })
        }

        dir
    }

    pub fn to_block_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();

        for item in &self.items {
            let temp = item.inode_pos.to_le_bytes();
            data.extend_from_slice(&temp);

            let name_len = (item.name.len() as u32).to_le_bytes();
            data.extend_from_slice(&name_len);

            let name = item.name.as_bytes();
            data.extend_from_slice(name);

            let typ_len = (item.typ.len() as u32).to_le_bytes();
            data.extend_from_slice(&typ_len);

            let typ = item.typ.as_bytes();
            data.extend_from_slice(typ);

            let size_data = item.size.to_le_bytes();
            data.extend_from_slice(&size_data);
        }

        let magic = 0xDEADBEAFu32.to_le_bytes();
        data.extend_from_slice(&magic);

        data
    }
    
    pub fn show(&self) {
        if !self.items.is_empty() {
            println!("---Name---\t---Type---\t---Size---");
        }
        for item in &self.items {
            println!("{:^10}\t{:^10}\t{:^10}", item.name, item.typ, item.size);
        }
    }

    pub fn init_dir(&mut self, parent_inode_index: usize) {
        self.items.push(DirItem {
            inode_pos: self.inode_index as u32,
            name: String::from("."),
            typ: String::from("dir"),
            size: 0,
        });
        self.items.push(DirItem {
            inode_pos: parent_inode_index as u32,
            name: String::from(".."),
            typ: String::from("dir"),
            size: 0,
        });
    }
}

#[derive(Debug)]
pub struct DirItem {
    pub inode_pos: u32,
    pub name: String,
    pub typ: String,
    pub size: u32,
}