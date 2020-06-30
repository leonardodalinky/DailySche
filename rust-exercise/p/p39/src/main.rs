// Exercise 39: Dictionaries

use std::collections::HashMap;

fn main() {
    let mut hm = HashMap::new();
    hm.insert(1i32, "Hello".to_string());
    hm.insert(0i32, "World".to_string());
    hm.insert(1i32, "Replace".to_string());
    for (i, s) in hm.iter() {
        println!("Index: {}, Value: {}", i, s);
    }
}
