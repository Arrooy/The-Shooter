use std::env;

use ext2::Ext2;
use fat16::FAT16;
use generics::*;

mod fat16;
mod ext2;
mod utils;
mod generics;

fn main() {
    // Extract the program arguments
    let operation = env::args().nth(1).expect("Operation is missing");
    let volume_name = env::args().nth(2).expect("Volume name arg is missing");
    let file_name = env::args().nth(3);


    // Create a new FileSystem
    let unknown_vol = GenericVolume::new(volume_name, file_name);
    let filesystem: Box<dyn Filesystem>;

    if unknown_vol.is_fat() {
        filesystem = Box::new(FAT16::new(unknown_vol))
    } else if unknown_vol.is_ext2() {
        filesystem = Box::new(Ext2::new(unknown_vol))
    } else {
        print!("{}", ERROR_VOLUME_FORMAT_NOT_RECOGNIZED);
        return;
    }

    filesystem.process_operation(operation)
}
