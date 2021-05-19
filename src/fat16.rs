use core::fmt;
use std::process::exit;
use std::fs;
use crate::generics::*;
use crate::utils::*;

//TODO: Potser els fitxers (directoris) no ocupen un sol cluster. S'ha de emprar la FAT per a saber si hi han més dades!

pub(crate) struct FAT16 {
    file_name: String,
    vol_name: String,
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
    fn check_fat_type_is_fat16(&self) {
        match self.get_fat_type() {
            FatType::FAT12 => {
                println!("{}", ERROR_FAT_12_FOUND);
                exit(-1);
            }
            FatType::FAT16 => return,
            FatType::FAT32 => {
                println!("{}", ERROR_FAT_32_FOUND);
                exit(-1);
            }
        }
    }

    fn get_fat_type(&self) -> FatType {
        return match self.data_sec / self.bpb_sec_per_clus as u32 {
            count_of_clusters if count_of_clusters < 4085 => FatType::FAT12,
            count_of_clusters if count_of_clusters < 65525 => FatType::FAT16,
            _ => FatType::FAT32
        };
    }

    // Cerca un directori per trobar query_filename. Al trobar una carpeta, torna a executar la cerca a l'interior.
    // Basicament és un DFS.
    fn find_in_dir(&self, start: u32, end: u32, query_filename: &String) -> Option<u32> {
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
            let nom = extract_string(directory, 0, 8).unwrap().replace(" ", "");
            let extension = extract_string(directory, 8, 3).unwrap().replace(" ", "");

            let filename = {
                if extension == "" {
                    nom.to_lowercase()
                } else {
                    format!("{}.{}", nom, extension).to_lowercase()
                }
            };

            let attr = directory[11];

            let cluster_numbers = (directory[27] as u16) << 8 | (directory[26] as u16);
            let file_size = extract_u32(directory, 28);

            // Directori es un subdirectori! Podem buscar a l'interior. Sempre i quan no sigui . o ..
            if attr == 0x10 && directory[0] != 0x2e {

                let first_sector_of_cluster = ((cluster_numbers - 2) as u32 * self.bpb_sec_per_clus as u32) + self.first_data_sector as u32;
                let new_dir_start = first_sector_of_cluster * self.bpb_byts_per_sec as u32;

                let res = self.find_in_dir(new_dir_start, 0, query_filename);
                if res.is_some() {
                    return res;
                }
            } else if *query_filename == filename {
                return Some(file_size);
            }
            i += 32;
        }

        return None;
    }

    // Delete a file in root dir.
    fn delete_in_dir(&self, start: u32, end: u32, query_filename: &String) -> Vec<u8> {
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
            let nom = extract_string(directory, 0, 8).unwrap().replace(" ", "");
            let extension = extract_string(directory, 8, 3).unwrap().replace(" ", "");

            let filename = {
                if extension == "" {
                    nom.to_lowercase()
                } else {
                    format!("{}.{}", nom, extension).to_lowercase()
                }
            };

            let attr = directory[11];
            let cluster_numbers = (directory[27] as u16) << 8 | (directory[26] as u16);

            let file_size = extract_u32(directory, 28);
            // Directori es un subdirectori! Podem buscar a l'interior. Sempre i quan no sigui . o ..
            if *query_filename == filename && !(attr == 0x10 && directory[0] != 0x2e){
                let mut new_data = self.data.to_vec();

                new_data[i as usize] = 0xE5;
                // Si el fitxer és pler, invalidem el contingut
                if file_size != 0 {
                    let first_sector_of_cluster = ((cluster_numbers - 2) as u32 * self.bpb_sec_per_clus as u32) + self.first_data_sector as u32;
                    let new_dir_start = first_sector_of_cluster * self.bpb_byts_per_sec as u32;
                    for k in new_dir_start as usize .. (new_dir_start + file_size) as usize {
                        new_data[k] = 0;
                    }
                }
                return new_data;
            }
            i += 32;
        }

        return vec![];
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

        let obj = FAT16 {
            vol_name:gv.vol_name,
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

        };
        obj.check_fat_type_is_fat16();
        obj
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
        let first_root_dir_sec_num = self.num_rsvd_sec as u32 + (self.bpb_num_fats as u32 * self.bpb_fatsz16 as u32);
        let first_root_dir_start = first_root_dir_sec_num * self.bpb_byts_per_sec as u32;
        let first_root_dir_end = (self.root_dir_sectors as u32 * self.bpb_byts_per_sec as u32) + first_root_dir_start;

        let file_size = self.find_in_dir(first_root_dir_start, first_root_dir_end, &self.file_name);
        if file_size.is_some() {
            println!("{}{} bytes.", FILE_FOUND, file_size.unwrap());
        } else {
            println!("{}", FILE_NOT_FOUND);
        }
    }

    fn delete(&self) {
        let first_root_dir_sec_num = self.num_rsvd_sec as u32 + (self.bpb_num_fats as u32 * self.bpb_fatsz16 as u32);
        let first_root_dir_start = first_root_dir_sec_num * self.bpb_byts_per_sec as u32;
        let first_root_dir_end = (self.root_dir_sectors as u32 * self.bpb_byts_per_sec as u32) + first_root_dir_start;

        let edited_file: Vec<u8> = self.delete_in_dir(first_root_dir_start, first_root_dir_end, &self.file_name);
        if edited_file.len() != 0 {
            fs::write(format!("{}{}", RESOURCES_PATH, self.vol_name), edited_file).expect("Unable to save new filesystem! Check program permissions!");
            println!("{}{}{}", FILE_DELETED_1, self.file_name, FILE_DELETED_2);
        } else {
            println!("{}", FILE_NOT_FOUND);
        }
    }
}
