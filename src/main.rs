use std::fs;
use std::env;
use std::borrow::Borrow;
use std::fmt::Display;

const RESOURCES_PATH: &str = "./res/";

const ERROR_VOLUME_NOT_FOUND: &str = "Error! No s'ha trobat el volum";

const ERROR_NUM_ARGS: &str = "Error amb el nombre d'arguments. El programa espera 2 o 3 arguments.\
Per l'informació d'un volum prova: cargo run /info vol_name\
Per buscar un fitxer prova: cargo run /find vol_name file_name.txt";

const ERROR_OPTION_NOT_FOUND: &str = "Opcio no reconeguda! Opcions reconegudes són /info /find /delete";


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

struct Volume{
    data: Vec<u8>,
    file_name: Option<String>,
}

impl Filesystem for Volume{
    fn new(volume_name: String, file_name: Option<String>) -> Self {
        Self {
            data: fs::read(format!("{}{}", RESOURCES_PATH, volume_name)).expect(format!("{}{}",ERROR_VOLUME_NOT_FOUND, volume_name).as_str()),
            file_name
        }
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
    let filesystem= Volume::new(volume_name, file_name);

    filesystem.process_operation(operation)
}
