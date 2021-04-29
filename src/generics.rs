use std::fs;

use crate::utils::extract_string;

pub(crate) const RESOURCES_PATH: &str = "./res/";

pub(crate) const ERROR_VOLUME_NOT_FOUND: &str = "Error. Volum inexistent";

pub(crate) const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

pub(crate) const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";

pub(crate) const ERROR_VOLUME_FORMAT_NOT_RECOGNIZED: &str = "Error. Volum no formatat en FAT16 ni EX2.";

pub(crate) const INFO_HEADER: &str = "------ Filesystem Information ------";

pub struct GenericVolume {
    pub(crate) data: Vec<u8>,
    pub(crate) file_name: Option<String>,
}

impl GenericVolume {
    pub(crate) fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}", ERROR_VOLUME_NOT_FOUND).as_str()),
            file_name,
        }
    }

    pub(crate) fn is_fat(&self) -> bool {
        if self.data.len() >= 511 {
            // BS_FilSysType contains FAT (One of the strings “FAT12 ”, “FAT16 ”, or “FAT ”.) in position 54.
            // Check sector 510 == 0x55 and sector 511 == 0xAA
            extract_string(&self.data, 54, 8).unwrap().contains("FAT") && self.data[510..=511] == [0x55, 0xAA]
        } else {
            false
        }
    }

    pub(crate) fn is_ext2(&self) -> bool {
        if self.data.len() >= 1081 {
            // Mirem el magic number del filesystem
            self.data[1024 + 56..=1024 + 57] == [0x53, 0xEF]
        } else {
            false
        }
    }
}

pub(crate) trait Filesystem {
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
