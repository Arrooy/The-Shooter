use crate::generics::*;
use crate::utils::*;
use std::borrow::Borrow;

pub(crate) struct Ext2 {
    file_name: Option<String>,
    data: Vec<u8>,

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
    fn getOffset(&self, block_number: u32) -> u32 {
        return 1024 + (block_number - 1) * self.block_size;
    }

}

impl Filesystem for Ext2 {
    fn new(gv: GenericVolume) -> Self {
        Ext2 {
            file_name: gv.file_name,

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

        println!("Bloc size is {:?}", self.block_size);


        // Read group descriptors info:
        let block_group_count = 1 + (self.block_count - 1) / self.group_blocks_count;

        //Offset del primer bloc.
        let gr_desc_start_off = self.getOffset(1 + self.first_block);

        //Offset del final del block group description table
        let gr_desc_end_off = gr_desc_start_off + block_group_count * 32;


        // Inode table
        let inodes_per_block = self.block_size / self.inode_size as u32;
        let inode_blocks_per_group = self.inodes_x_group / inodes_per_block;

        let mut inode_num = 1 + self.first_inode;
        let bg_inode_table = extract_u32(&self.data,(gr_desc_start_off + 8) as usize);

        while inode_num < self.inodes_x_group {
            let in_table_start_off = (self.getOffset(bg_inode_table) + (inode_num - 1) * self.inode_size as u32) as usize;
            let i_mode = extract_u16(&self.data,in_table_start_off);

            // TODO: Preguntar que fer amb els inodes que donen 0...
            let i_links_count = extract_u16(&self.data, in_table_start_off + 26);
            if i_mode != 0 && i_links_count > 0 {

                let max_i_blocks = extract_u32(&self.data, in_table_start_off + 28) / (2 << extract_u32(&self.data, 1024 + 24));
                if (i_mode & 0x4000) == 0x4000 {
                    println!("El inode és un directori!");
                }else {

                }
                // Aixo 15 vegades. i_block_0 apunta a
                let i_block_0 = extract_u32(&self.data, in_table_start_off + 40);

                println!("I block is {} - {}", i_block_0,i_block_1);
                println!("Scanning inode.Start of the table is {:x}. Format is {:x} Its size is {:?}. Its i_blocks is {:?}",in_table_start_off, i_mode, extract_u32(&self.data,in_table_start_off + 4), max_i_blocks);

            }
            inode_num += 1;
        }
    }
    fn delete(&self) {
        todo!()
    }
}
