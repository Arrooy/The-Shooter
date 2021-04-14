use std::fs;
use std::env;

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error! No s'ha trobat el volum";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";


struct Volume<'a> {
    data: Vec<u8>,
    file_name: &'a str,
}

trait Filesystem{
    fn new(volume_name: &'static str, file_name: &'static str) -> Volume<'static> {
        Volume {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}{}",ERROR_VOLUME_NOT_FOUND, volume_name).as_str()),
            file_name
        }
    }

    fn process_operation(&self, operation: &'static str) {
        match operation {
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

struct FAT16<'a>(Volume<'a>);

impl Filesystem for FAT16<'_>{

    fn info(&self){

    }

    fn find(&self, file_name: &'static str) {
        todo!()
    }

    fn delete(&self, file_name: &'static str) {
        todo!()
    }
}

struct EX2<'a>(Volume<'a>);
impl EX2 <'_>{

}

fn main() {
    // Process the program arguments
    let args: Vec<String> = env::args().collect();

    let filesystem;

    match args.len() {
        // Volem fer info
        3 => filesystem = FAT16::new(args[2].as_str(), ""),
        // Volem fer funcionalitats amb fitxers
        4 => filesystem = FAT16::new(args[2].as_str(), args[3].as_str()),
        _ => {
            print!("{}",ERROR_NUM_ARGS);
            return;
        },
    }

    filesystem.process_operation(args[1].as_str())
}
