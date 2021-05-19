use std::cmp::min;

use crate::generics::*;
use crate::utils::*;
use std::process::exit;
use std::fs;

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

    indirect_block_row_count: u32,
    double_indirect_top_row: u32,


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

struct FindResult{
    file_size:u32,
    file_inode:usize,
}

impl Ext2 {
    fn get_offset(&self, block_number: usize) -> usize {
        return if self.first_block == 1 {
            // Block number == 1 -> Superblock.
            1024 + (block_number - 1) * self.block_size as usize
        } else {
            block_number * self.block_size as usize
        };
    }

    // Calcula quin es el index maxim a l'interor del array de blocks del indode.
    fn compute_max_index(&self, max_i_blocks: u32) -> u32{
        return if max_i_blocks < 13 {
            max_i_blocks
        } else if max_i_blocks >= 13 && max_i_blocks <= self.indirect_block_row_count - 12 {
            13
        } else if max_i_blocks > self.indirect_block_row_count - 12 && max_i_blocks <= self.double_indirect_top_row {
            14
        } else {
            15
        }
    }

    // Proporciona el offset desde l'inici del filesystem fins a l'inici de la taula d'inodes d'un inode concret.
    fn compute_inode_table_start_offset(&self, inode_num: usize) -> usize{
        let block_group_num = (inode_num - 1) / self.inodes_x_group as usize;
        let block_group_offset = block_group_num * self.group_blocks_count as usize * self.block_size as usize;

        let bg_inode_table = {
            if block_group_num == 0 || block_group_num == 1 || is_power(block_group_num, 3) || is_power(block_group_num, 5) || is_power(block_group_num, 7) {
                // Proporciona el block de la taula d'inodes del primer block group.
                extract_u32(&self.data, self.get_offset(1 + self.first_block as usize + block_group_num * self.group_blocks_count as usize) + 8) as usize
            } else {
                // Si el block_group_num no és una potencia de 3, 5, 7 -> NO HI HA SUPERBLOCK NI BLOCK GROUP. Nomes cal saltar els 2 bitmaps.
                2 * self.block_size as usize
            }
        };

        return block_group_offset + self.get_offset(bg_inode_table);
    }


    // Donat un inode_num proporciona el offset a la seva posició a memoria desde l'inici del fs.
    fn compute_inode_offset(&self, global_inode_num: usize) -> usize {
        let relative_inode_num = (global_inode_num - 1) % self.inodes_x_group as usize;

        // Donat un inode_num proporciona el offset a la seva posició desde l'inici de tot el fs.
        return self.compute_inode_table_start_offset(global_inode_num) + relative_inode_num * self.inode_size as usize;
    }


    // Cerca tots els blocks a linterior d'un block indirecte. Si troba el fitxer, retorna la mida.
    // layer especifica la profunditat. Si és 0, estem tractant single-indirect block.
    // Sino, cal endinsarse més.
    fn explore_indirect_block(&self, indirect_block_offset: u32, filename: &String, max_loop: u32, layer: u32, delete_directory_entry: bool) -> Option<FindResult> {
        println!("Exploring indirect block: {} {:x} MaxLoop is {} and layer is {}", indirect_block_offset, indirect_block_offset, max_loop, layer);
        let mut k = 0;
        while {
            let block_number_inside_indirect_block = extract_u32(&self.data, (indirect_block_offset + 4 * k) as usize);

            let mut data_valid = block_number_inside_indirect_block != 0;

            if data_valid {
                let data_block_offset = block_number_inside_indirect_block * self.block_size;

                let file_size = {
                    if layer == 0 {
                        self.find_in_dir(data_block_offset, filename, delete_directory_entry)
                    } else {
                        // Les seguents layers iterem per tots els valors!
                        self.explore_indirect_block(data_block_offset, filename, 0, layer - 1, delete_directory_entry)
                    }
                };

                if file_size.is_some() {
                    return file_size;
                }
            }
            // Si maxloop es 0, iterem fins a trobar un valor 0 en les dades
            // Sino, iterem fins a max_loop.
            if max_loop != 0 && k >= max_loop {
                data_valid = false;
            }

            k = k + 1;
            data_valid
        } {}
        None
    }

    // Cerca un fitxer de forma recursiva, retorna el Some(FindResult). Si no troba. retorna None.
    // Find result conté mida de fitxer i inode d'aquest.
    fn find_in_inode(&self, inode_num: usize, filename: &String, delete_directory_entry: bool) -> Option<FindResult> {

        // Donat un inode_num proporciona el offset a la seva posició.
        let offset = self.compute_inode_offset(inode_num);

        let i_mode = extract_u16(&self.data, offset);

        // El inode_num correspon a un directori.
        if (i_mode & 0x4000) == 0x4000 {
            let max_i_blocks = extract_u32(&self.data, offset + 28) / (2 << extract_u32(&self.data, 1024 + 24));

            // Computa quin es l'index màxim del array.
            let max_loop = self.compute_max_index(max_i_blocks);

            for i in 1..=max_loop {
                // println!("i is {}", i);
                if i < 13 {
                    // Direct blocks: Els offsets son correctes, no fa falta fer gaire...
                    let data_block_offset = extract_u32(&self.data, offset + 40 + 4 * (i - 1) as usize) * self.block_size;

                    let file_size = self.find_in_dir(data_block_offset, filename, delete_directory_entry);

                    if file_size.is_some() {
                        return file_size;
                    }
                } else {
                    // Pot ser simple - double o triple indirect.
                    let indirect_block_offset = extract_u32(&self.data, offset + 40 + 4 * (i - 1) as usize) * self.block_size;
                    let indirect_max_loop = {
                        if i == 13 {
                            min(max_i_blocks, self.indirect_block_row_count - 12)
                        } else if i == 14 {
                            min(max_i_blocks, self.double_indirect_top_row)
                        } else {
                            // Iterem a muerte fins trobar una entrada amb valor 0.
                            0
                        }
                    };

                    let file_size = self.explore_indirect_block(indirect_block_offset, filename, indirect_max_loop, i - 13, delete_directory_entry);

                    if file_size.is_some() {
                        return file_size;
                    }
                }
            }
        } else {
            // S'executa quan hem trobat el inode del fitxer.
            println!("File inode is {} Offset is dec: {} hex: {:x}", inode_num, offset, offset);
            return Some(FindResult {
                file_size: extract_u32(&self.data, offset + 4),
                file_inode: inode_num,
            });
        }
        None
    }

    // Analitza un directori que ocupa 1 sol bloc.
    fn find_in_dir(&self, data_block_offset: u32, filename: &String, delete_directory_entry: bool) -> Option<FindResult> {
        let mut i: usize = 0;
        // Do while suuper apurat.
        while {
            let goal_inode = extract_u32(&self.data, data_block_offset as usize + i) as usize;

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
                        // println!("File found: {} Finding in inode {}", found_filename, goal_inode);
                        let size = self.find_in_inode(goal_inode, filename, delete_directory_entry);
                        if size.is_some() {
                            return size;
                        }
                    }
                }
            } else {
                // Is a directory. Recursive call.

                //Evitem analitzar . i ..
                if found_filename == "." || found_filename == ".." {
                    // println!("Not analizing dir search because its me! Filename is {} Reclen is {}", found_filename, rec_len);
                } else {
                    // println!("Analizing a inode {} that is a dir!! Dirname is {} filetype is {}", goal_inode, found_filename, file_type);
                    let size = self.find_in_inode(goal_inode, filename, delete_directory_entry);
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

    //http://manpages.ubuntu.com/manpages/precise/man8/e2undel.8.html apartat NOTES.
    fn delete_inode(&self, file_inode: usize) -> Vec<u8> {
        let mut new_data = self.data.to_vec();

        // ---- Alliberar els nodes dels bitmaps ----

        // Computem la posicio de la taula d'inodes del inode a borrar.
        let inode_table_start_offset = self.compute_inode_table_start_offset(file_inode);

        // Restem 1 block a la localitzacio de la taula per arribar al inode bitmap.
        let inode_bitmap_offset = inode_table_start_offset - 1 * self.block_size as usize;
        let relative_inode_num = (file_inode - 1) % self.inodes_x_group as usize;
        let bitmap_byte_num = inode_bitmap_offset + relative_inode_num / 8;

        set_bit(&mut new_data,  bitmap_byte_num,(relative_inode_num % 8) as u8);

        // Restem 2 block a la localitzacio de la taula per arribar al block bitmap.
        let data_block_bitmap_offset = inode_table_start_offset - 2 * self.block_size as usize;


        // ---- Modificar delete time "d_time" ----

        // Donat un inode_num proporciona el offset a la seva posició.
        let offset = self.compute_inode_offset(file_inode);

        // Es modifica el camp d_time
        let time = current_time();
        save_u32(&mut new_data, offset + 20, time);

        // Retornem les dades modificades.
        return vec![];
    }
}

impl Filesystem for Ext2 {
    fn new(gv: GenericVolume) -> Self {

        let block_size = extract_log_u32(&gv.data, 1024 + 24);
        // Quantitat de rows dels blocs indirectes. Emprat a find i delete.
        let indirect_block_row_count =  block_size / 4;

        Ext2 {
            file_name: gv.file_name,
            vol_name: gv.vol_name,

            indirect_block_row_count,
            double_indirect_top_row: indirect_block_row_count - 12 + block_size * block_size,

            inode_count: extract_u32(&gv.data, 1024),
            free_inodes: extract_u32(&gv.data, 1024 + 16),
            inodes_x_group: extract_u32(&gv.data, 1024 + 40),
            first_inode: extract_u32(&gv.data, 1024 + 84),
            inode_size: extract_u16(&gv.data, 1024 + 88),

            block_size,
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
        let found_result = self.find_in_inode(2, &self.file_name, false);

        if found_result.is_some() {
            println!("{}{} bytes.", FILE_FOUND, found_result.unwrap().file_size);
        } else {
            println!("{}", FILE_NOT_FOUND);
        }
    }
    fn delete(&self) {

        // TODO: Fer que find_in_inode rebi una clousure indicant que ha de tetornar. La clousure reb
        // Iniciem la cerca per el inode Root.

        let found_result = self.find_in_inode(2, &self.file_name, true);

        if found_result.is_some() {
            // Hem trobat el inode! Borrem!
            let edited_file: Vec<u8> =self.delete_inode(found_result.unwrap().file_inode);
            if edited_file.len() != 0 {
                fs::write(format!("{}{}", RESOURCES_PATH, self.vol_name), edited_file).expect("Unable to save new filesystem! Check program permissions!");
                println!("{}{}{}", FILE_DELETED_1, self.file_name, FILE_DELETED_2);
            } else {
                println!("{}", FILE_NOT_FOUND);
            }
        } else {
            println!("{}", FILE_NOT_FOUND);
        }
    }
}
