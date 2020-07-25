#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

#[no_mangle]
pub fn main() -> usize {
    println!("Hello world from user mode program!");
    //println!("Clone id is {}", user_lib::sys_clone());
    //println!("Syscall: The thread id of hello-world is {}.", user_lib::sys_gettid());

    // open a file
    let test_fd = user_lib::sys_open("test");
    println!("test_fd is {}", test_fd);
    let mut buffer = [0u8;32];
    user_lib::sys_read(test_fd as usize, &mut buffer);
    println!("{:?}", buffer);
    0
}
