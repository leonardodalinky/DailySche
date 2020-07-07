use std::collections::HashMap;

#[derive(Debug)]
struct LRUCache {
    cap: i32,
    size: i32,
    order: Vec<i32>,
    num: HashMap<i32, i32>
}


/**
 * `&self` means the method takes an immutable reference.
 * If you need a mutable reference, change it to `&mut self` instead.
 */
impl LRUCache {

    fn new(capacity: i32) -> Self {
        Self {
            cap: capacity,
            size: 0,
            order: Vec::new(),
            num: HashMap::new()
        }
    }

    fn get(&mut self, key: i32) -> i32 {
        match self.num.get(&key) {
            None => -1,
            Some(i) => {
                let newi = *i;
                let mut index = 0;
                for (it, x) in self.order.iter().zip(0..self.order.len()) {
                    if *it == key {
                        index = x;
                        break;
                    }
                }
                self.order.remove(index as usize);
                self.order.push(key);

                newi
            }
        }
    }

    fn put(&mut self, key: i32, value: i32) {
        if self.size < self.cap {

            match self.num.get(&key) {
                None => {
                    self.size += 1;
                    self.order.push(key);
                    self.num.insert(key, value);
                },
                Some(i) => {
                    let mut index = 0;
                    for (it, x) in self.order.iter().zip(0..self.order.len()) {
                        if *it == key {
                            index = x;
                            break;
                        }
                    }
                    let out = self.order.remove(index as usize);
                    self.order.push(key);
                    self.num.remove(&out);
                    self.num.insert(key, value);
                }
            };
        }
        else {
            let mut index = 0;
            for (it, x) in self.order.iter().zip(0..self.order.len()) {
                if *it == key {
                    index = x;
                    break;
                }
            }
            let out = self.order.remove(index as usize);
            self.order.push(key);
            self.num.remove(&out);
            self.num.insert(key, value);
        }
    }
}

/**
 * Your LRUCache object will be instantiated and called as such:
 * let obj = LRUCache::new(capacity);
 * let ret_1: i32 = obj.get(key);
 * obj.put(key, value);
 */

fn main() {
    let mut obj = LRUCache::new(3);
    obj.put(2, 3);
    obj.put(4, 1);
    obj.put(4, 2);
    obj.put(1, 2);
    println!("{:?}", obj);
}
