#[cfg(test)]
use std::process::Command;
#[cfg(test)]
use rand::{self, Rng};
#[cfg(test)]
use std::env::temp_dir;
#[cfg(test)]
use std::ffi::{OsString};
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::prelude::*;
#[cfg(test)]
use std::str;

macro_rules! try_opt(
    ($e:expr) => (match $e { Some(e) => e, None => return None })
);

macro_rules! try_opt_void(
    ($e:expr) => (match $e { Some(e) => e, None => return })
);

#[inline(always)]
pub fn bits(b: u16, start: u8, len: u8) -> u8 {
    bits16(b, start, len) as u8
}
#[inline(always)]
pub fn bits16(b: u16, start: u8, len: u8) -> u16 {
    (b >> start & ((1 << len) - 1))
}

#[inline(always)]
pub fn bit(b: u8, pos: usize) -> u8 {
    (b >> pos) & 1
}

#[inline(always)]
pub fn bit16(b: u16, pos: usize) -> u8 {
    ((b >> pos) & 1) as u8
}

#[inline(always)]
pub fn bitneg(b: u8, pos: usize) -> u8 {
    !bit(b, pos) & 1
}

#[inline(always)]
pub fn bitneg16(b: u16, pos: usize) -> u8 {
    !bit16(b, pos) & 1
}


#[cfg(test)]
pub fn assemble_to_file(code: &str) -> OsString {
    let input_name = tmpfile();
    let middle_name = tmpfile();
    let output_name = tmpfile();

    let mut input_file = File::create(input_name.clone()).unwrap();
    input_file.write_all(code.as_bytes()).unwrap();

    let o = Command::new("avr-gcc")
        .arg("-s")
        .arg("-o")
        .arg(middle_name.clone())
        .arg("-Wa,-mmcu=atmega32")
        .arg(input_name)
        .output()
        .expect("avr-gcc failed");

    if !o.status.success() {
        println!("avr-gcc: {:?}, {:?}",
                 str::from_utf8(&o.stdout).unwrap(),
                 str::from_utf8(&o.stderr).unwrap());
    }

    let o = Command::new("avr-objcopy")
        .arg("-O").arg("binary")
        .arg(middle_name)
        .arg(output_name.clone())
        .output()
        .expect("avr-objcopy failed");

    if !o.status.success() {
        println!("avr-objcopy: {:?}, {:?}",
                 str::from_utf8(&o.stdout).unwrap(),
                 str::from_utf8(&o.stderr).unwrap());
    }

    output_name
}

#[cfg(test)]
#[allow(dead_code)]
pub fn assemble(code: &str) -> Vec<u8> {
    let mut output = File::open(assemble_to_file(code)).unwrap();
    let mut assembled = Vec::new();
    output.read_to_end(&mut assembled).unwrap();
    // println!("Assembled: {:?}", assembled);
    assembled
}

#[cfg(test)]
fn tmpfile() -> OsString {
    let s = rand::thread_rng()
        .gen_ascii_chars()
        .take(10)
        .collect::<String>();
    let mut file = temp_dir();
    file.push("vm-".to_string() + &s + ".s");
    file.into_os_string()
}
