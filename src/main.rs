use std::fs;
use std::env;
use std::str;
use std::borrow::Borrow;
use std::fmt::Display;
use std::fs::File;
use std::str::Utf8Error;

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error! No s'ha trobat el volum";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";

const ERROR_VOLUME_FORMAT_NOT_RECOGNIZED: &str = "Error. No es possible reconeixer el format del volum";


enum SupportedFilesystems {
    FAT16,
    EX2,
}

trait Filesystem{
    fn new(volume_name: String, file_name: Option<String>) -> Self;

    fn process_operation(&self, operation: String) {
        match operation.as_str() {
            "/info" => self.info(),
            "/find" => todo!(),
            "/delete" => todo!(),
            _ => print!("{}",ERROR_OPTION_NOT_FOUND),
        }
    }

    fn info(&self);
    fn find(&self, file_name: &'static str);
    fn delete(&self, file_name: &'static str);
}

struct GenericVolume {
    data: Vec<u8>,
    file_name: Option<String>
}

impl GenericVolume{
    fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}{}",ERROR_VOLUME_NOT_FOUND, volume_name).as_str()),
            file_name
        }
    }
    // print!("{:#04x}",&self.data[82..=90]);
    // print!("{:?}",&self.data[3..=8]);
    // Box<dyn Filesystem>

    fn is_fat(&self) -> bool{
        // BS_FilSysType contains FAT (One of the strings “FAT12 ”, “FAT16 ”, or “FAT ”.) in position 54.
        // Check sector 510 == 0x55 and sector 511 == 0xAA
        extract(&self.data, 54, 8).unwrap().contains("FAT") && self.data[510..=511] == [0x55,0xAA]
    }

    fn is_ex2(&self) -> bool{
        todo!()
    }
}

fn extract(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error>  {
    str::from_utf8(&data[base..=base+offset])
}

struct FAT16 {
    data: Vec<u8>,
    file_name: Option<String>,
    bpb_byts_per_sec: u16,
    bpb_sec_per_clus: u8,
    bpb_rsvd_sec_cnt: u16,
    bpb_num_fats: u8,
    bpb_root_ent_cnt: u16,
    bpb_tot_sec16: u16, // Un dels dos ha de ser non zero!
    bpb_tot_sec32: u32,  // Un dels dos ha de ser non zero!
    bs_vol_lab: String,
}

impl Filesystem for FAT16 {
    //TODO: Crear constructor desde genericVol.
    fn new(volume_name: String, file_data: Option<String>) -> Self {
       todo!()
    }

    fn info(&self){

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
    let filesystem: dyn Filesystem;

    if unkown_vol.is_fat() {
        filesystem = FAT16::new(unkown_vol.)
    }else if unkown_vol.is_ex2() {
        todo!()
    }else{
        print!(ERROR_VOLUME_FORMAT_NOT_RECOGNIZED)
    }

    filesystem.process_operation(operation)
}
