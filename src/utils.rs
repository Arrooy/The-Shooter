extern crate chrono;

use std::str;
use std::str::Utf8Error;

use chrono::prelude::*;
use std::time::{UNIX_EPOCH, SystemTime};

//https://www.geeksforgeeks.org/check-if-a-number-is-power-of-another-number/
pub(crate) fn is_power(x: usize, y:u64) -> bool{
    let mut x = x as f64;
    let y = y as f64;

    while x % y == 0.0 {
        x = x / y
    }
    return x == 1.0
}

pub(crate) fn extract_string(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error> {
    str::from_utf8(&data[base..base + offset])
}

// Extrau un string fins a trobar un \0
pub(crate) fn extract_string_terminated(data: &[u8], base: usize, offset: usize) -> Result<&str, Utf8Error> {
    let vec = &data[base..base + offset];

    // Si la cadena son tot 0, retornem un valor indicatiu de que no hi ha res.
    if vec.iter().all(|&x| x == 0) {
        Ok("<Not defined>")
    } else {
        Ok(str::from_utf8(vec).unwrap().split("\0").collect::<Vec<_>>()[0])
    }
}

pub(crate) fn extract_u16(data: &[u8], base: usize) -> u16 {
    let vec = &data[base..base + 2];
    //println!("Extracting [{}..{}] is {:?}", base, 2, vec);
    ((vec[1] as u16) << 8) | vec[0] as u16
}

pub(crate) fn extract_u32(data: &[u8], base: usize) -> u32 {
    let vec = &data[base..base + 4];
    //println!("Extracting [{}..{}] is {:?}", base, 4, vec);
    ((vec[3] as u32) << 24) | ((vec[2] as u32) << 16) | ((vec[1] as u32) << 8) | (vec[0] as u32)
}

pub(crate) fn save_u32(data: &mut [u8], base: usize, new_data: u32) {
    data[base] = (new_data & 0xff) as u8;
    data[base + 1] = ((new_data >> 8) & 0xff) as u8;
    data[base + 2] = ((new_data >> 16) & 0xff) as u8;
    data[base + 3] = ((new_data >> 24) & 0xff) as u8;
}

pub(crate) fn set_bit(data: &mut [u8], base: usize, bit_number: u8){
    data[base] = data[base] & !(1 << bit_number);
}

pub(crate) fn extract_log_u32(data: &[u8], base: usize) -> u32 {
    1024 << extract_u32(data, base)
}

pub(crate) fn timestamp_to_date_time(timestamp: u32) -> String {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    let time: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    time.format("%a %b %e %T %Y").to_string()
}

pub(crate) fn current_time() -> u32 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards :)").as_secs() as u32
}