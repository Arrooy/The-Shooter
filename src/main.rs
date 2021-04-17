use std::borrow::Borrow;
use std::env;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::str;
use std::str::Utf8Error;

/*
    Fat analysis
    fatcat -i Fat16_1024

    Ex2 analysis
    dumpe2fs -h Ext2

*/

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error! No s'ha trobat el volum";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";

const ERROR_VOLUME_FORMAT_NOT_RECOGNIZED: &str = "Error. No es possible reconeixer el format del volum";

const INFO_HEADER: &str = "------ Filesystem Information ------";

struct GenericVolume {
    data: Vec<u8>,
    file_name: Option<String>,
}

impl GenericVolume {
    fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}{}", ERROR_VOLUME_NOT_FOUND, volume_name).as_str()),
            file_name,
        }
    }
    // print!("{:#04x}",&self.data[82..=90]);
    // print!("{:?}",&self.data[3..=8]);
    // Box<dyn Filesystem>

    fn is_fat(&self) -> bool {
        // BS_FilSysType contains FAT (One of the strings “FAT12 ”, “FAT16 ”, or “FAT ”.) in position 54.
        // Check sector 510 == 0x55 and sector 511 == 0xAA
        extract_string(&self.data, 54, 8).unwrap().contains("FAT") && self.data[510..=511] == [0x55, 0xAA]
    }

    fn is_ex2(&self) -> bool {
        todo!()
    }
}

fn extract_string(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error> {
    str::from_utf8(&data[base..base + offset])
}

fn extract_u16(data: &[u8], base: usize, offset: usize) -> u16 {
    let vec = &data[base..base + offset];
    println!("Extracting [{}..{}] is {:?}", base, offset, vec);
    ((vec[0] as u16) << 8) | vec[1] as u16
}

fn extract_u32(data: &[u8], base: usize, offset: usize) -> u32 {
    let vec = &data[base..base + offset];
    println!("Extracting [{}..{}] is {:?}", base, offset, vec);
    ((vec[0] as u32) << 24) | ((vec[1] as u32) << 16) | ((vec[2] as u32) << 8) | (vec[3] as u32)
}

trait Filesystem {
    fn new(gv: GenericVolume) -> Self;

    fn process_operation(&self, operation: String) {
        match operation.as_str() {
            "/info" => self.info(),
            "/find" => todo!(),
            "/delete" => todo!(),
            _ => print!("{}", ERROR_OPTION_NOT_FOUND),
        }
    }

    fn info(&self);
    fn find(&self, file_name: &'static str);
    fn delete(&self, file_name: &'static str);
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
        if extract_u32(&gv.data, 32, 4) == 0 {
            num_total_sec = extract_u16(&gv.data, 19, 2) as u32;
        } else {
            num_total_sec = extract_u32(&gv.data, 32, 4);
        }

        FAT16 {
            file_name: gv.file_name,
            bytes_per_sector: extract_u16(&gv.data, 11, 2),
            num_sec_per_alloc: gv.data[13],
            num_rsvd_sec: extract_u16(&gv.data, 14, 2),
            num_fats: gv.data[16],
            num_root_dir: extract_u16(&gv.data, 17, 2),
            num_total_sec,
            bs_vol_lab: extract_string(&gv.data, 43, 11).unwrap().parse().unwrap(),
            oem_name: extract_string(&gv.data, 3, 8).unwrap().parse().unwrap(),
            data: gv.data,
        }
    }

    fn info(&self) {
        print!("{}
Filesystem: FAT16
System Name: {}
Mida del sector: {}
Sectors Per Cluster: {}
Sectors reservats: {}
Número de FATs): {}
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

    fn find(&self, file_name: &'static str) {
        todo!()
    }
    fn delete(&self, file_name: &'static str) {
        todo!()
    }
}

fn main() {
    // Extract the program arguments
    let operation = env::args().nth(1).expect("");
    let volume_name = env::args().nth(2).expect("");
    let file_name = env::args().nth(3);

    // Create a new FileSystem
    let unkown_vol = GenericVolume::new(volume_name, file_name);
    let filesystem;

    if unkown_vol.is_fat() {
        filesystem = FAT16::new(unkown_vol)
    } else if unkown_vol.is_ex2() {
        filesystem = FAT16::new(unkown_vol)
    } else {
        panic!("{}", ERROR_VOLUME_FORMAT_NOT_RECOGNIZED)
    }

    filesystem.process_operation(operation)
}
