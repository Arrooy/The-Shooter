use std::env;
use std::process::exit;

use ext2::Ext2;
use fat16::FAT16;
use generics::*;

mod fat16;
mod ext2;
mod utils;
mod generics;


fn main() {
    let (operation, volume_name, file_name) = process_args();

    // Create a new generic FileSystem
    let unknown_vol = GenericVolume::new(volume_name, file_name);
    let filesystem: Box<dyn Filesystem>;

    // Create an instance based on its type.
    if unknown_vol.is_fat() {
        filesystem = Box::new(FAT16::new(unknown_vol));
    } else if unknown_vol.is_ext2() {
        filesystem = Box::new(Ext2::new(unknown_vol));
    } else {
        println!("{}", ERROR_VOLUME_FORMAT_NOT_RECOGNIZED);
        exit(-1);
    }

    // Satisfy the user needs.
    filesystem.process_operation(operation);
}


// Extract the program arguments
fn process_args() -> (String, String, String) {
    let operation = {
        let arg = env::args().nth(1);
        if arg.is_none() {
            exit_with_params_error();
        }
        arg.unwrap()
    };

    //Check que el nombre de args és el correcte basat en l'operation.
    if operation == "/info" {
        if env::args().count() != 3 {
            exit_with_params_error();
        }
    } else {
        if env::args().count() != 4 {
            exit_with_params_error();
        }
    }

    let volume_name = {
        let arg = env::args().nth(2);
        if arg.is_none() {
            exit_with_params_error();
        }
        arg.unwrap()
    };

    let file_name = {
        let arg = env::args().nth(3);
        if arg.is_some() {
            if operation == "/info" {
                exit_with_params_error();
            } else {
                arg.unwrap()
            }
        } else {
            if operation != "/info" {
                println!("File name not specified!");
                exit(-1);
            } else {
                return (operation, volume_name, String::new());
            }
        }
    };

    (operation, volume_name, file_name)
}

fn exit_with_params_error() -> ! {
    println!("{}", ERROR_NUM_PARAMS_WRONG);
    exit(-1);
}