extern crate chrono;

use std::str;
use std::str::Utf8Error;

use chrono::prelude::*;

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

pub(crate) fn extract_log_u32(data: &[u8], base: usize) -> u32 {
    1024 << extract_u32(data, base)
}

pub(crate) fn timestamp_to_date_time(timestamp: u32) -> String {
    let naive_datetime = NaiveDateTime::from_timestamp(timestamp as i64, 0);
    let time: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
    time.format("%a %b %e %T %Y").to_string()
}