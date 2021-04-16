use std::borrow::Borrow;
use std::env;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::str;
use std::str::Utf8Error;

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
    ((vec[0] as u16) << 8) | vec[1] as u16
}

fn extract_u32(data: &[u8], base: usize, offset: usize) -> u32 {
    let vec = &data[base..base + offset];
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
    bpb_byts_per_sec: u16,
    bpb_sec_per_clus: u8,
    bpb_rsvd_sec_cnt: u16,
    bpb_num_fats: u8,
    bpb_root_ent_cnt: u16,
    bpb_tot_sec16: u16,
    // Un dels dos ha de ser non zero!
    bpb_tot_sec32: u32,
    // Un dels dos ha de ser non zero!
    bs_vol_lab: String,
    bs_oemname: String,
    data: Vec<u8>,
}

impl Filesystem for FAT16 {
    fn new(gv: GenericVolume) -> Self {
        FAT16 {
            file_name: gv.file_name,

            bpb_byts_per_sec: extract_u16(&gv.data, 11, 2),
            bpb_sec_per_clus: gv.data[13],
            bpb_rsvd_sec_cnt: extract_u16(&gv.data, 14, 2),
            bpb_num_fats: gv.data[16],
            bpb_root_ent_cnt: extract_u16(&gv.data, 17, 2),
            bpb_tot_sec16: extract_u16(&gv.data, 19, 2),
            bpb_tot_sec32: extract_u32(&gv.data, 32, 4),
            bs_vol_lab: extract_string(&gv.data, 54, 8).unwrap().parse().unwrap(),
            bs_oemname: extract_string(&gv.data, 3, 8).unwrap().parse().unwrap(),
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
Número de FATs (16): {}
Número de FATs (32): {}
MaxRootEntries: {}
Sectors per FAT: {}
Label: {}",
            //TODO: REORDENAR!
               INFO_HEADER,
               self.bs_oemname,
               self.bpb_byts_per_sec,
               self.bpb_sec_per_clus,
               self.bpb_rsvd_sec_cnt,
               self.bpb_num_fats,
               self.bpb_root_ent_cnt,
               self.bpb_tot_sec16,
               self.bpb_tot_sec32,
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
