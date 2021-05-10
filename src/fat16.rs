use core::fmt;

use crate::generics::*;
use crate::utils::*;

pub(crate) struct FAT16 {
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
    first_data_sector: u32,
    data_sec: u32,
}
//TODO: QUE PASSA SI EL SIZE ES ZERO?

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

impl FAT16 {
    fn get_fat_type(&self) -> FatType {
        return match self.data_sec / self.bpb_sec_per_clus as u32 {
            count_of_clusters if count_of_clusters < 4085 => FatType::FAT12,
            count_of_clusters if count_of_clusters < 65525 => FatType::FAT16,
            _ => FatType::FAT32
        };
    }

    fn get_sec(&self, n: u32) -> u32 {
        let fat_offset = n * 2;

        let fat_sec_num = self.num_rsvd_sec as u32 + (fat_offset / self.bpb_byts_per_sec as u32);
        let fat_ent_offset = fat_offset % self.bpb_byts_per_sec as u32;
        print!("Fat offset is {}", fat_ent_offset);
        12
    }
    // Cerca un directori per trobar query_filename. Al trobar una carpeta, torna a executar la cerca a l'interior.
    // Basicament és un DFS.
    fn find_in_dir(&self, start: u32, end: u32, query_filename: &String) -> u32 {
        let mut i: u32 = start;
        while i < end || end == 0 {
            let directory = &self.data[i as usize..(i + 32) as usize];

            // No hi ha info en aquest bloc. Anem al seguent
            if directory[0] == 0xE5 || directory[11] == 15 {
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
            let filename = (nom.unwrap().replace(" ", "") + "." + extension.unwrap()).replace(" ", "").to_lowercase();
            let attr = directory[11];

            let cluster_numbers = (directory[27] as u16) << 8 | (directory[26] as u16);
            let file_size = extract_u32(directory, 28);

            // TODO: Podria ser que el filename sigui el nom de la carpeta. I que no trobi el fitxer perque troba abans la carpeta?
            // TODO: Hauria doncs de mirar si el tipo es fitxer apart del nom?
            if *query_filename == filename {
                return file_size;
            }

            // Directori es un subdirectori! Podem buscar a l'interior. Sempre i quan no sigui . o ..
            if attr == 0x10 && directory[0] != 0x2e {
                // println!("El directori era una carpeta. Entrant a l'interior de la carpeta {}", filename);
                let first_sector_of_cluster = ((cluster_numbers - 2) as u32 * self.bpb_sec_per_clus as u32) + self.first_data_sector as u32;
                let new_dir_start = first_sector_of_cluster * self.bpb_byts_per_sec as u32;

                let res = self.find_in_dir(new_dir_start, 0, query_filename);
                if res != 0 {
                    return res;
                }
            }
            i += 32;
        }

        return 0;
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

        let bpb_tot_sec16 = extract_u16(&gv.data, 19);

        let bpb_tot_sec = if bpb_tot_sec16 == 0 {
            extract_u32(&gv.data, 32)
        } else {
            bpb_tot_sec16 as u32
        };

        // Count of sectors in data region
        let data_sec: u32 = bpb_tot_sec - first_data_sector;

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

        let first_root_dir_sec_num = self.num_rsvd_sec as u32 + (self.bpb_num_fats as u32 * self.bpb_fatsz16 as u32);
        let first_root_dir_start = first_root_dir_sec_num * self.bpb_byts_per_sec as u32;
        let first_root_dir_end = (self.root_dir_sectors as u32 * self.bpb_byts_per_sec as u32) + first_root_dir_start;

        let size = self.find_in_dir(first_root_dir_start, first_root_dir_end, query_filename);
        if size != 0 {
            println!("Fitxer trobat! Ocupa {} bytes.", size);
        } else {
            println!("Error. Fitxer no trobat.");
        }
    }

    fn delete(&self) {
        todo!()
    }
}
