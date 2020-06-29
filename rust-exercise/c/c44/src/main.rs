// Exercise 44. Ring Buffer

mod ring_buffer;

use ring_buffer::RingBuffer;

fn main() {
    let mut r = RingBuffer::new(5);
    r.write(5).unwrap();
    r.write(7).unwrap();
    println!("{}", r.read().unwrap());
}
