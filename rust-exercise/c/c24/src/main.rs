// Exercise 24. Input, Output, Files
// Simplified version.

use std::io::{stdin};

fn main() {
    println!("Please input something:");
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    println!("You write: {}", s);
}
