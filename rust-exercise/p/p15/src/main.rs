// Exercise 15: Reading Files

use std::env;
use std::fs::File;
use std::io::{Read, stdin, Write};
use std::path::Path;

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("Please input a filepath!");
    }
    let s = args.nth(1).unwrap();
    let path = Path::new(&s).file_name();
    println!("Your file name is: {}", path.unwrap().to_str().unwrap());
}
