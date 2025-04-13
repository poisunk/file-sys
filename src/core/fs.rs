use crate::core::dir::Dir;
use crate::core::dir::DirItem;
use crate::core::file::File;
use crate::core::hardware;
use crate::core::hardware::Hardware;
use crate::core::inode::Inode;

const INIT_MAGIC: u32 = 0xDEADBEEF_u32;

#[derive(Debug)]
pub struct System {
    pub initialized: bool,
    pub root_inode_index: usize,
    pub free_inodes: Vec<bool>,
    pub free_blocks: Vec<bool>,
    pub inodes: Vec<Inode>,
    pub hardware: Hardware,
}

impl System {
    pub fn init(hardware: Hardware) -> Self {
        let mut instance = Self {
            initialized: false,
            root_inode_index: 0,
            free_inodes: Vec::new(),
            free_blocks: Vec::new(),
            inodes: Vec::new(),
            hardware: hardware,
        };

        instance.load_super_block();

        instance.load_inodes();
        instance.load_free_blocks();
        instance.load_free_inodes();

        if !instance.initialized {
            instance.init_root_dir();
        }

        instance
    }

    pub fn get_root_dir(&self) -> Dir {
        let root_dir_inode = &self.inodes[self.root_inode_index];
        let inode_data = self.read_inode_data(self.root_inode_index);
        let root_dir =
            Dir::from_block_bytes(&root_dir_inode.name, self.root_inode_index, &inode_data);
        root_dir
    }

    pub fn open_dir(&self, dir: &Dir, name: &str) -> Dir {
        let root_dir_inode = &self.inodes[dir.inode_index];
        let inode_data = self.read_inode_data(dir.inode_index);
        let root_dir = Dir::from_block_bytes(&root_dir_inode.name, dir.inode_index, &inode_data);

        let target_inode_index = root_dir
            .items
            .iter()
            .filter(|item| item.name == name)
            .next()
            .unwrap()
            .inode_pos as usize;
        let target_inode = &self.inodes[target_inode_index];
        let inode_data = self.read_inode_data(target_inode_index);
        Dir::from_block_bytes(&target_inode.name, target_inode_index, &inode_data)
    }

    pub fn create_dir(&mut self, dir: &mut Dir, name: &str) -> Dir {
        for item in dir.items.iter() {
            if item.name == name {
                todo!("Dir already exists");
            }
        }

        let free_inode_index = self.get_next_free_inode() as usize;
        self.inodes[free_inode_index].init(name);
        self.set_free_inode_used(free_inode_index, true);

        let mut target_dir = Dir::new(
            self.inodes[free_inode_index].name.as_str(),
            free_inode_index,
        );
        target_dir.init_dir(dir.inode_index);

        let data = target_dir.to_block_bytes();

        self.write_with_inode(free_inode_index, &data);

        dir.items.push(DirItem {
            inode_pos: free_inode_index as u32,
            name: name.to_string(),
            typ: "dir".to_string(),
            size: 0,
        });

        let root_data = dir.to_block_bytes();
        self.write_with_inode(dir.inode_index, &root_data);

        target_dir
    }

    pub fn remove_dir(&mut self, root: &mut Dir, name: &str) {
        let mut target_dir = root
            .items
            .iter()
            .filter(|item| item.name == name && item.typ == "dir")
            .next()
            .map(|item| {
                let inode_data = self.read_inode_data(item.inode_pos as usize);
                Dir::from_block_bytes(item.name.as_str(), item.inode_pos as usize, &inode_data)
            })
            .unwrap();

        root.items
            .retain(|item| item.inode_pos != target_dir.inode_index as u32);

        let root_data = root.to_block_bytes();
        self.write_with_inode(root.inode_index, &root_data);

        for i in 0..target_dir.items.len() {
            let item_name = target_dir.items[i].name.clone();
            let item_type = target_dir.items[i].typ.clone();

            if item_name == "." || item_name == ".." {
                continue;
            }

            if item_type == "dir" {
                self.remove_dir(&mut target_dir, &item_name);
            } else if item_type == "file" {
                self.remove_file(&mut target_dir, &item_name);
            }
        }

        self.remove_inode_data(target_dir.inode_index);
    }

    pub fn create_file(&mut self, dir: &mut Dir, name: &str) -> File {
        let free_inode_index = self.get_next_free_inode() as usize;
        self.set_free_inode_used(free_inode_index, true);
        self.inodes[free_inode_index].init(name);

        let target_file = File::new(name, free_inode_index);
        dir.items.push(DirItem {
            inode_pos: free_inode_index as u32,
            name: name.to_string(),
            typ: "file".to_string(),
            size: 0,
        });

        let root_data = dir.to_block_bytes();
        self.write_with_inode(dir.inode_index, &root_data);

        target_file
    }

    pub fn write_file(&mut self, dir: &mut Dir, file: &mut File, data: &[u8]) {
        self.write_with_inode(file.inode_index, data);
        file.content = String::from_utf8(data.to_vec()).unwrap();
        file.size = data.len() as u32;

        for i in 0..dir.items.len() {
            if dir.items[i].name == file.name {
                dir.items[i].size = data.len() as u32;
            }
        }

        let root_data = dir.to_block_bytes();
        self.write_with_inode(dir.inode_index, &root_data);
    }

    pub fn read_file(&self, file: &File) -> Vec<u8> {
        self.read_inode_data(file.inode_index)
            .iter()
            .filter(|x| **x != 0)
            .map(|x| *x)
            .collect()
    }

    pub fn remove_file(&mut self, dir: &mut Dir, name: &str) {
        let target_inode_index = dir
            .items
            .iter()
            .filter(|item| item.name == name && item.typ == "file")
            .next()
            .map(|item| item.inode_pos as usize)
            .unwrap();

        self.remove_inode_data(target_inode_index);
        self.set_free_inode_used(target_inode_index, false);
        dir.items.retain(|item| item.inode_pos != target_inode_index as u32);

        let root_data = dir.to_block_bytes();
        self.write_with_inode(dir.inode_index, &root_data);
    }

    pub fn open_file(&mut self, dir: &mut Dir, name: &str) -> File {
        if let Some(item) = dir.items.iter().filter(|item| item.name == name).next() {
            let inode_data = self.read_inode_data(item.inode_pos as usize);
            File::from_block_bytes(item.name.as_str(), item.inode_pos as usize, &inode_data)
        }else {
            self.create_file(dir, name)
        }
    }

    fn load_super_block(&mut self) {
        let magic = u32::from_le_bytes(self.hardware.data[0..4].try_into().unwrap());

        self.initialized = magic == INIT_MAGIC;

        self.root_inode_index =
            u32::from_le_bytes(self.hardware.data[4..8].try_into().unwrap()) as usize;

    }

    fn init_root_dir(&mut self) {
        assert!(self.inodes.len() > 0);
        self.set_free_inode_used(self.root_inode_index, true);

        self.inodes[self.root_inode_index].name = String::from("/");

        let mut root_dir = Dir::new(
            self.inodes[self.root_inode_index].name.as_str(),
            self.root_inode_index,
        );
        root_dir.init_dir(self.root_inode_index);

        let data = root_dir.to_block_bytes();

        self.write_with_inode(self.root_inode_index, &data);
    }

    fn load_free_blocks(&mut self) {
        let start_i = hardware::BLOCK_SIZE;
        self.free_blocks = self.hardware.data[start_i..start_i + hardware::BLOCK_SIZE - 3]
            .iter()
            .map(|x| *x == 1)
            .collect();
    }

    fn load_free_inodes(&mut self) {
        assert!(self.inodes.len() > 0);
        let start_i = hardware::BLOCK_SIZE * 2;
        self.free_inodes = self.hardware.data[start_i..start_i + self.inodes.len()]
            .iter()
            .map(|x| *x == 1)
            .collect();
    }

    fn load_inodes(&mut self) {
        let r = hardware::BLOCK_SIZE * 3;
        let l = r + hardware::BLOCK_SIZE;
        self.inodes = Inode::from_block_bytes(&self.hardware.data[r..l]);
    }

    fn get_next_free_block(&mut self) -> u32 {
        for i in 4..self.free_blocks.len() {
            if !self.free_blocks[i] {
                return i as u32;
            }
        }
        todo!()
    }

    fn get_next_free_inode(&mut self) -> u32 {
        for i in 0..self.free_inodes.len() {
            if !self.free_inodes[i] {
                return i as u32;
            }
        }
        todo!()
    }

    fn set_free_inode_used(&mut self, inode_pos: usize, used: bool) {
        self.free_inodes[inode_pos] = used;
    }

    fn set_free_block_used(&mut self, block_pos: usize, used: bool) {
        self.free_blocks[block_pos] = used;
    }

    fn write_with_inode(&mut self, inode_pos: usize, data: &[u8]) {
        for i in 0..self.inodes[inode_pos].block_pos.len() {
            self.clean_block_data(self.inodes[inode_pos].block_pos[i] as usize);
            self.set_free_block_used(self.inodes[inode_pos].block_pos[i] as usize, false);
        }

        let size = data.len();
        let mut positions: Vec<u32> = data
            .chunks(hardware::BLOCK_SIZE)
            .map(|chunk| {
                let free_block_index = self.get_next_free_block();
                self.set_free_block_used(free_block_index as usize, true);
                self.write_into_block(free_block_index as usize, chunk);
                free_block_index
            })
            .collect();

        if positions.len() > 7 {
            todo!()
        } else {
            positions.resize(7, 0);
        }

        self.inodes[inode_pos].size = size as u32;
        self.inodes[inode_pos].block_pos = positions;
    }

    fn write_into_block(&mut self, block_pos: usize, data: &[u8]) {
        if data.len() > hardware::BLOCK_SIZE {
            todo!()
        }
        let r = block_pos * hardware::BLOCK_SIZE;
        let l = r + data.len();
        self.hardware.data[r..l].copy_from_slice(data);
    }

    fn read_inode_data(&self, inode_pos: usize) -> Vec<u8> {
        let mut data = Vec::new();
        for block_pos in self.inodes[inode_pos].block_pos.iter() {
            if block_pos == &0 {
                break;
            }
            let r = *block_pos as usize * hardware::BLOCK_SIZE;
            let l = r + hardware::BLOCK_SIZE;
            data.extend_from_slice(&self.hardware.data[r..l]);
        }
        data
    }

    fn remove_inode_data(&mut self, inode_pos: usize) {
        for i in 0..self.inodes[inode_pos].block_pos.len() {
            if self.inodes[inode_pos].block_pos[i] == 0 {
                break;
            }
            let block_pos = self.inodes[inode_pos].block_pos[i];
            self.clean_block_data(block_pos as usize);
            self.set_free_block_used(block_pos as usize, false);
        }

        self.set_free_inode_used(inode_pos, false);

        self.inodes[inode_pos].clean();
    }

    fn clean_block_data(&mut self, block_pos: usize) {
        self.hardware.data
            [block_pos * hardware::BLOCK_SIZE..(block_pos + 1) * hardware::BLOCK_SIZE]
            .copy_from_slice(&[0; hardware::BLOCK_SIZE]);
    }

    pub fn save(&mut self, name: &str) {
        let free_block_data = self
            .free_blocks
            .iter()
            .map(|x| if *x { 0x01 } else { 0x00 })
            .collect::<Vec<u8>>();
        self.write_into_block(1, &free_block_data);

        let free_inode_data = self
            .free_inodes
            .iter()
            .map(|x| if *x { 0x01 } else { 0x00 })
            .collect::<Vec<u8>>();
        self.write_into_block(2, &free_inode_data);

        let inodes_data = self
            .inodes
            .iter()
            .map(|x| x.to_le_bytes())
            .into_iter()
            .flatten()
            .collect::<Vec<u8>>();
        self.write_into_block(3, &inodes_data);

        self.hardware.data[..4].copy_from_slice(INIT_MAGIC.to_le_bytes().as_slice());

        self.hardware.save(name);
    }
}
