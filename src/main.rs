extern crate chrono;

use std::env;
use std::fs;
use std::str;
use std::str::Utf8Error;

use chrono::prelude::*;

/*
    Fat analysis
    fatcat -i Fat16_1024

    Ex2 analysis
    dumpe2fs -h Ext2

*/

// TODO: FAT16 no concorda el camp SectorsxFAT
// TODO: preguntar si es pot fer la conversio de temps amb libs.

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error! No s'ha trobat el volum";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";

const ERROR_VOLUME_FORMAT_NOT_RECOGNIZED: &str = "Sistema d’arxius no és ni EXT2 ni FAT16";

const INFO_HEADER: &str = "------ Filesystem Information ------";

struct GenericVolume {
    data: Vec<u8>,
    file_name: Option<String>,
}

impl GenericVolume {
    fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{} {}", ERROR_VOLUME_NOT_FOUND, volume_name).as_str()),
            file_name,
        }
    }

    fn is_fat(&self) -> bool {
        if self.data.len() >= 511 {
            // BS_FilSysType contains FAT (One of the strings “FAT12 ”, “FAT16 ”, or “FAT ”.) in position 54.
            // Check sector 510 == 0x55 and sector 511 == 0xAA
            extract_string(&self.data, 54, 8).unwrap().contains("FAT") && self.data[510..=511] == [0x55, 0xAA]
        }else{
            false
        }
    }

    fn is_ext2(&self) -> bool {
        if self.data.len() >= 1081 {
            // Mirem el magic number del filesystem
            self.data[1024 + 56..=1024 + 57] == [0x53, 0xEF]
        }else{
            false
        }
    }
}

fn extract_string(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error> {
    str::from_utf8(&data[base..base + offset])
}
// Extrau un string fins a trobar un \0
fn extract_string_terminated(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error> {
    let vec =  &data[base..base + offset];

    // Si la cadena son tot 0, retornem un valor indicatiu de que no hi ha res.
    if vec.iter().all(|&x| x == 0) {
        Ok("<Not defined>")
    }else{
        Ok(str::from_utf8(vec).unwrap().split("\0").collect::<Vec<_>>()[0])
    }
}


fn extract_u16(data: &[u8], base: usize) -> u16 {
    let vec = &data[base..base + 2];
    // println!("Extracting [{}..{}] is {:?}", base, 2, vec);
    ((vec[1] as u16) << 8) | vec[0] as u16
}

fn extract_u32(data: &[u8], base: usize) -> u32 {
    let vec = &data[base..base + 4];
    // println!("Extracting [{}..{}] is {:?}", base, 4, vec);
    ((vec[3] as u32) << 24) | ((vec[2] as u32) << 16) | ((vec[1] as u32) << 8) | (vec[0] as u32)
}

fn extract_log_u32(data: &[u8], base: usize) -> u32 {
    1024 << extract_u32(data, base)
}

fn timestamp_to_date_time(timestamp: u32) -> String {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    let time: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    time.format("%a %b %e %T %Y").to_string()
}

trait Filesystem {
    fn new(gv: GenericVolume) -> Self
        where Self: Sized;

    fn process_operation(&self, operation: String) {
        match operation.as_str() {
            "/info" => self.info(),
            "/find" => self.find(),
            "/delete" => self.delete(),
            _ => print!("{}", ERROR_OPTION_NOT_FOUND),
        }
    }

    fn info(&self);
    fn find(&self);
    fn delete(&self);
}

struct FAT16 {
    file_name: Option<String>,
    bytes_per_sector: u16,
    num_sec_per_alloc: u8,
    num_rsvd_sec: u16,
    num_fats: u8,
    num_root_dir: u16,
    num_total_sec: u32,
    bs_vol_lab: String,
    oem_name: String,
    data: Vec<u8>,
}

impl Filesystem for FAT16 {
    fn new(gv: GenericVolume) -> Self {
        let num_total_sec: u32;

        // Mirem si el nombre de sectors de 32 bits esta buit. Sino el fem servir.
        if extract_u32(&gv.data, 32) == 0 {
            num_total_sec = extract_u16(&gv.data, 19) as u32;
        } else {
            num_total_sec = extract_u32(&gv.data, 32);
        }

        FAT16 {
            file_name: gv.file_name,
            bytes_per_sector: extract_u16(&gv.data, 11),
            num_sec_per_alloc: gv.data[13],
            num_rsvd_sec: extract_u16(&gv.data, 14),
            num_fats: gv.data[16],
            num_root_dir: extract_u16(&gv.data, 17),
            num_total_sec,
            bs_vol_lab: extract_string(&gv.data, 43, 11).unwrap().parse().unwrap(),
            oem_name: extract_string(&gv.data, 3, 8).unwrap().parse().unwrap(),
            data: gv.data,
        }
    }

    fn info(&self) {
        println!("{}\n
Filesystem: FAT16
System Name: {}
Mida del sector: {}
Sectors Per Cluster: {}
Sectors reservats: {}
Número de FATs: {}
MaxRootEntries: {}
Sectors per FAT: {}
Label: {}",
               INFO_HEADER,
               self.oem_name,
               self.bytes_per_sector,
               self.num_sec_per_alloc,
               self.num_rsvd_sec,
               self.num_fats,
               self.num_root_dir,
               self.num_total_sec,
               self.bs_vol_lab)
    }

    fn find(&self) {
        todo!()
    }
    fn delete(&self) {
        todo!()
    }
}

struct Ext2 {
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
        todo!()
    }
    fn delete(&self) {
        todo!()
    }
}


fn main() {
    // Extract the program arguments
    let operation = env::args().nth(1).expect("Operation is missing");
    let volume_name = env::args().nth(2).expect("Volume name arg is missing");
    let file_name = env::args().nth(3);


    // Create a new FileSystem
    let unkown_vol = GenericVolume::new(volume_name, file_name);
    let filesystem: Box<dyn Filesystem>;

    if unkown_vol.is_fat() {
        filesystem = Box::new(FAT16::new(unkown_vol))
    } else if unkown_vol.is_ext2() {
        filesystem = Box::new(Ext2::new(unkown_vol))
    } else {
        print!("{}", ERROR_VOLUME_FORMAT_NOT_RECOGNIZED);
        return;
    }

    filesystem.process_operation(operation)
}
