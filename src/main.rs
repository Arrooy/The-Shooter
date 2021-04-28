extern crate chrono;

use std::env;
use std::fs;
use std::str;
use std::str::Utf8Error;

use chrono::prelude::*;
use core::fmt;
/*
    Fat analysis
    fatcat -i Fat16

    Ex2 analysis
    dumpe2fs -h Ext2

*/

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error. Volum inexistent";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";

const ERROR_VOLUME_FORMAT_NOT_RECOGNIZED: &str = "Error. Volum no formatat en FAT16 ni EX2.";

const INFO_HEADER: &str = "------ Filesystem Information ------";

struct GenericVolume {
    data: Vec<u8>,
    file_name: Option<String>,
}

impl GenericVolume {
    fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}", ERROR_VOLUME_NOT_FOUND).as_str()),
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
    //println!("Extracting [{}..{}] is {:?}", base, 2, vec);
    ((vec[1] as u16) << 8) | vec[0] as u16
}

fn extract_u32(data: &[u8], base: usize) -> u32 {
    let vec = &data[base..base + 4];
    //println!("Extracting [{}..{}] is {:?}", base, 4, vec);
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
    bpb_byts_per_sec: u16,
    bpb_sec_per_clus: u8,
    num_rsvd_sec: u16,
    bpb_num_fats: u8,
    bpb_root_ent_cnt: u16,
    bpb_fatsz16: u16,
    bs_vol_lab: String,
    oem_name: String,
    data: Vec<u8>,
    root_dir_sectors: u16,
    first_data_sector:u32,
    data_sec: u32,
}

enum FatType {
    FAT12,
    FAT16,
    FAT32,
}

impl fmt::Display for FatType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FatType::FAT12 => write!(f, "FAT12"),
            FatType::FAT16 => write!(f, "FAT16"),
            FatType::FAT32 => write!(f, "FAT32"),
        }
    }
}

impl FAT16{
    fn get_fat_type(&self) -> FatType {

        return match self.data_sec / self.bpb_sec_per_clus as u32 {
            count_of_clusters if count_of_clusters < 4085 => FatType::FAT12,
            count_of_clusters if count_of_clusters <  65525 => FatType::FAT16,
            _ =>  FatType::FAT32
        }
    }

    fn get_sec(&self, n: u32) -> u32{
        let fat_offset = n * 2;

        let fat_sec_num = self.num_rsvd_sec as u32 + (fat_offset / self.bpb_byts_per_sec as u32);
        let fat_ent_offset = fat_offset % self.bpb_byts_per_sec as u32;
        print!("Fat offset is {}", fat_ent_offset);
        12
    }
}

impl Filesystem for FAT16 {
    fn new(gv: GenericVolume) -> Self {

        let bpb_root_ent_cnt = extract_u16(&gv.data, 17);
        let bpb_byts_per_sec = extract_u16(&gv.data, 11);
        let num_rsvd_sec = extract_u16(&gv.data, 14);
        let bpb_num_fats = gv.data[16];
        let bpb_fatsz16 = extract_u16(&gv.data, 22);

        // Calcul del nombre de sectors que ocupa el root directory.
        let root_dir_sectors = ((bpb_root_ent_cnt * 32) + (bpb_byts_per_sec - 1)) / bpb_byts_per_sec;
        // Start of data sector
        let first_data_sector = num_rsvd_sec as u32 + (bpb_num_fats as u32 * bpb_fatsz16 as u32) + root_dir_sectors as u32;


        let bpb_tot_sec16 =  extract_u16(&gv.data, 19);

        let bpb_tot_sec = if bpb_tot_sec16 == 0 {
            extract_u32(&gv.data, 32)
        }else{
            bpb_tot_sec16 as u32
        };

        // Count of sectors in data region
        let data_sec: u32 =  bpb_tot_sec - first_data_sector;

        FAT16 {
            file_name: gv.file_name,
            bpb_byts_per_sec,
            bpb_sec_per_clus: gv.data[13],
            num_rsvd_sec,
            bpb_num_fats,
            bpb_root_ent_cnt,
            bpb_fatsz16,
            bs_vol_lab: extract_string(&gv.data, 43, 11).unwrap().parse().unwrap(),
            oem_name: extract_string(&gv.data, 3, 8).unwrap().parse().unwrap(),
            data: gv.data,
            root_dir_sectors,
            first_data_sector,
            data_sec,
        }
    }

    fn info(&self) {


            println!("{}\n
Filesystem: {}
System Name: {}
Mida del sector: {}
Sectors Per Cluster: {}
Sectors reservats: {}
Número de FATs: {}
MaxRootEntries: {}
Sectors per FAT: {}
Label: {}",
                     INFO_HEADER,
                    self.get_fat_type(),
                     self.oem_name,
                     self.bpb_byts_per_sec,
                     self.bpb_sec_per_clus,
                     self.num_rsvd_sec,
                     self.bpb_num_fats,
                     self.bpb_root_ent_cnt,
                     self.bpb_fatsz16,
                     self.bs_vol_lab)


    }

    fn find(&self) {

        if let None = self.file_name {
            panic!("File name not specified!")
        }

        let query_filename = self.file_name.as_ref().unwrap();

        match self.get_fat_type() {
            FatType::FAT12 => panic!("Filesystem must be FAT16. FAT12 found instead!"),
            FatType::FAT16 => (),
            FatType::FAT32 => println!("Filesystem must be FAT16. FAT32 found instead!"),
        }

        let first_root_dir_sec_num = self.num_rsvd_sec as u32 + ( self.bpb_num_fats as u32 * self.bpb_fatsz16 as u32);
        let first_root_dir_start = first_root_dir_sec_num * self.bpb_byts_per_sec as u32;
        let first_root_dir_end = (self.root_dir_sectors as u32 * self.bpb_byts_per_sec as u32) + first_root_dir_start;

        let mut i :u32 = first_root_dir_start;
        let mut found:bool = false;

        while i < first_root_dir_end {
            let directory = &self.data[i as usize..(i+32) as usize];

            // No hi ha info en aquest bloc. Anem al seguent
            if directory[0] == 0xE5 {
                i += 32;
                continue;
            }

            // No hi ha info en el bloc ni en en el seguents
            if directory[0] == 0x00 {
                break;
            }

            // Hem trobat un directori!
            let nom = extract_string(directory, 0, 8);
            let extension = extract_string(directory, 8, 3);
            let filename = (nom.unwrap().replace(" ","") + "." + extension.unwrap()).to_lowercase();
            let attr = directory[11];

            /* Check nom llarg.
            if attr == 0xf{
                print!("El nom es LONG NAME!")
            }
             */

            let cluster_numbers = (directory[26] as u16) << 8 | (directory[27] as u16);
            let file_size = extract_u32(directory,28);

            if *query_filename == filename {
                println!("Fitxer trobat! Ocupa {} bytes.", file_size);
                found = true;
                break;
            }
            //println!("Info found:\n {:?} -- {:?} {:?} {:#04x} {:?} {:?}", filename, nom , extension, attr, cluster_numbers, file_size);
            i += 32;
        }
        if !found {
            println!("Error. Fitxer no trobat.")
        }
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
