use std::{cmp::Reverse, collections::BinaryHeap};
use std::cmp::Ordering;

#[derive(Copy, Clone)]
struct Item(i32);

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&(other.0)).reverse()
    }
}

impl Eq for Item {

}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.0.partial_cmp(&other.0) {
            None => None,
            Some(e) => Some(e.reverse()),
        }
    }
}

pub fn find_kth_largest(nums: Vec<i32>, k: i32) -> i32 {
    let mut q: BinaryHeap<Item> = BinaryHeap::new();
    let len = nums.len();
    for it in nums.iter() {
        if q.len() < k as usize {
            q.push(Item(*it));
        }
        else {
            let top = q.peek().unwrap();
            if *it > top.0 {
                q.pop();
                q.push(Item(*it));
            }
        }
    }

    q.pop().unwrap().0
}

fn main() {
    let r = find_kth_largest(vec![1,2,3], 2);
    println!("{}", r);
}
