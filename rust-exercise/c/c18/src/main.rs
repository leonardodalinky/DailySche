// Exercise 18. Pointers to Functions
use std::env;

fn main() {
    // parse int
    let envs: Vec<String> = env::args().collect();
    let mut ints: Vec<i32> = Vec::new();
    for i in &envs[1..] {
        ints.push(i.parse::<i32>().unwrap());
    }
    bubble_sort(&mut ints, sorted_order);
    println!("{:?}", ints);
}

fn sorted_order(a: i32, b: i32) -> i32 {
    a - b
}

fn bubble_sort(ints: &mut Vec<i32>, comp: fn(i32, i32) -> i32) {
    let count = ints.len();
    for _i in 0..count {
        for j in 0..count-1 {
            if comp(ints[j], ints[j + 1]) > 0 {
                let tmp = ints[j + 1];
                ints[j + 1] = ints[j];
                ints[j] = tmp;
            }
        }
    }
}