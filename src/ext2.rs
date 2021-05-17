use crate::generics::*;
use crate::utils::*;

// Al fer delete, sha de tocar Block Group Descriptor Table?
// En l'inode, que s'ha de fer? modificar la dtime? Modificar la mida?
// S'han de tocar els blocs?

// Eliminar el bit del bitmap dels inodes.
// Eliminar lentrada del directory entry i linode.
// Posant bits a zero.

pub(crate) struct Ext2 {
    file_name: String,
    data: Vec<u8>,
    vol_name: String,

    inode_size: u16,
    inode_count: u32,
    first_inode: u32,
    inodes_x_group: u32,
    free_inodes: u32,

    block_size: u32,
    block_count: u32,
    rsvd_blocks: u32,
    free_blocks: u32,
    first_block: u32,
    group_blocks_count: u32,
    group_frags_count: u32,

    volume_name: String,

    last_check: u32,
    last_mount: u32,
    last_write: u32,
}

impl Ext2 {
    // Block number == 1 -> Superblock.
    fn get_offset(&self, block_number: u32) -> u32 {
        return 1024 + (block_number - 1) * self.block_size;
    }

    // Cerca un fitxer de forma recursiva, retorna el Some(size). Si no troba. retorna None.
    fn find_in_inode(&self, inode_nun: u32, filename: &String) -> Option<u32> {
        let block_group_num = (inode_nun - 1) / self.inodes_x_group;

        let block_group_offset = block_group_num * self.group_blocks_count * self.block_size;

        // Proporciona el blockd e la taula d'inodes del primer block group.
        let bg_inode_table = extract_u32(&self.data, (self.get_offset(1 + self.first_block + block_group_num * self.group_blocks_count) + 8) as usize);

        let inode_nun = (inode_nun - 1) % self.inodes_x_group;

        // Donat un inode_num proporciona el offset a la seva posició.
        let offset = (block_group_offset + self.get_offset(bg_inode_table) + inode_nun * self.inode_size as u32) as usize;

        let i_mode = extract_u16(&self.data, offset);
        if (i_mode & 0x4000) == 0x4000 {

            // El inode_num correspon a un directori.
            let max_i_blocks = extract_u32(&self.data, offset + 28) / (2 << extract_u32(&self.data, 1024 + 24));

            assert!(max_i_blocks <= 15);

            for i in 1..=max_i_blocks {
                let data_block_num = extract_u32(&self.data, offset + 40 + 4 * (i - 1) as usize);
                let data_block_offset = data_block_num * self.block_size;
                if i < 13 {
                    // Direct blocks
                    let file_size = self.find_in_dir(data_block_offset, filename);

                    if file_size.is_some() {
                        return file_size;
                    }
                } else if i == 13 {
                    // Indirect block
                    todo!()
                } else if i == 14 {
                    // Double indirect block
                    todo!()
                } else {
                    // Triple indirect block
                    todo!()
                }
            }
        } else {
            let size = extract_u32(&self.data, offset + 4);
            return Some(size);
        }
        None
    }

    // Analitza un directori que ocupa 1 sol bloc.
    fn find_in_dir(&self, data_block_offset: u32, filename: &String) -> Option<u32> {
        let mut i: usize = 0;
        // Do while suuper apurat.
        while {
            let goal_inode = extract_u32(&self.data, data_block_offset as usize + i);

            let name_len = &self.data[data_block_offset as usize + 6 + i];
            let found_filename = extract_string(&self.data, data_block_offset as usize + 8 + i, *name_len as usize).unwrap();
            let file_type = self.data[data_block_offset as usize + 7 + i];

            let rec_len = extract_u16(&self.data, data_block_offset as usize + 4 + i);

            // Si la entry no es fa servir + no hi ha seguent. Sortim de la recusio.
            if goal_inode == 0 && rec_len == 0 {
                return None;
            }

            // Check filetype:
            if file_type != 2 {
                // Not a dir. Check filename!
                if file_type != 0 {
                    if filename == found_filename {
                        let size = self.find_in_inode(goal_inode, filename);
                        if size.is_some() {
                            return size;
                        }
                    }
                }
            } else {
                // Is a directory. Recursive call.

                //Evitem analitzar . i ..
                if found_filename == "." || found_filename == ".." {
                    // println!("Not analizing dir search because its me! Filename is {} Reclen is {}",found_filename,rec_len);
                } else {
                    // println!("Analizing a inode {} that is a dir!! Dirname is {} filetype is {}",goal_inode, found_filename,file_type);
                    let size = self.find_in_inode(goal_inode, filename);
                    if size.is_some() {
                        return size;
                    }
                }
            }

            // Add the rec_len to the iterator
            i += rec_len as usize;

            //Condicio d'exit del doWhile apanyat. Si el rec_len + indeex actual > block_size, retornem
            (extract_u16(&self.data, data_block_offset as usize + 4 + i) as usize + i) <= self.block_size as usize
        } {}
        None
    }
}

impl Filesystem for Ext2 {
    fn new(gv: GenericVolume) -> Self {
        Ext2 {
            file_name: gv.file_name,
            vol_name:gv.vol_name,

            inode_count: extract_u32(&gv.data, 1024),
            free_inodes: extract_u32(&gv.data, 1024 + 16),
            inodes_x_group: extract_u32(&gv.data, 1024 + 40),
            first_inode: extract_u32(&gv.data, 1024 + 84),
            inode_size: extract_u16(&gv.data, 1024 + 88),

            block_size: extract_log_u32(&gv.data, 1024 + 24),
            block_count: extract_u32(&gv.data, 1024 + 4),
            rsvd_blocks: extract_u32(&gv.data, 1024 + 8),
            free_blocks: extract_u32(&gv.data, 1024 + 12),
            first_block: extract_u32(&gv.data, 1024 + 20),
            group_blocks_count: extract_u32(&gv.data, 1024 + 32),
            group_frags_count: extract_u32(&gv.data, 1024 + 36),

            volume_name: extract_string_terminated(&gv.data, 1024 + 120, 16).unwrap().parse().unwrap(),

            last_check: extract_u32(&gv.data, 1024 + 64),
            last_mount: extract_u32(&gv.data, 1024 + 44),
            last_write: extract_u32(&gv.data, 1024 + 48),

            data: gv.data,
        }
    }

    fn info(&self) {
        println!("{}\n
Filesystem: EXT2\n
INFO INODE
Mida Inode: {}
Num Inodes: {}
Primer Inode: {}
Inodes Grup: {}
Inodes Lliures: {}\n
INFO BLOC
Mida Bloc: {}
Blocs Reservats: {}
Blocs Lliures: {}
Total Blocs: {}
Primer Bloc: {}
Blocs grup: {}
Frags grup: {}\n
INFO VOLUM
Nom volum: {}
Ultima comprov: {}
Ultim muntatge: {}
Ultima escriptura: {}", INFO_HEADER,
                 self.inode_size,
                 self.inode_count,
                 self.first_inode,
                 self.inodes_x_group,
                 self.free_inodes,
                 self.block_size,
                 self.rsvd_blocks,
                 self.free_blocks,
                 self.block_count,
                 self.first_block,
                 self.group_blocks_count,
                 self.group_frags_count,
                 self.volume_name,
                 timestamp_to_date_time(self.last_check),
                 timestamp_to_date_time(self.last_mount),
                 timestamp_to_date_time(self.last_write),
        )
    }

    fn find(&self) {

        // Iniciem la cerca per el inode Root.
        let found_size = self.find_in_inode(2, &self.file_name);

        if found_size.is_some() {
            println!("{}{} bytes.", FILE_FOUND, found_size.unwrap());
        } else {
            println!("{}", FILE_NOT_FOUND);
        }
    }
    fn delete(&self) {
        todo!()
    }
}
