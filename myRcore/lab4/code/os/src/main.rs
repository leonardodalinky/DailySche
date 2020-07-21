//! # 全局属性
//!
//! - `#![no_std]`
//!   禁用标准库
#![no_std]
//!
//! - `#![no_main]`
//!   不使用 `main` 函数等全部 Rust-level 入口点来作为程序入口
#![no_main]
//!
//! - `#![deny(missing_docs)]`
//!   任何没有注释的地方都会产生警告：这个属性用来压榨写实验指导的学长，同学可以删掉了
#![warn(missing_docs)]
//! # 一些 unstable 的功能需要在 crate 层级声明后才可以使用
//!
//! - `#![feature(alloc_error_handler)]`
//!   我们使用了一个全局动态内存分配器，以实现原本标准库中的堆内存分配。
//!   而语言要求我们同时实现一个错误回调，这里我们直接 panic
#![feature(alloc_error_handler)]
//!
//! - `#![feature(llvm_asm)]`
//!   内嵌汇编
#![feature(llvm_asm)]
#![feature(asm)]
//!
//! - `#![feature(global_asm)]`
//!   内嵌整个汇编文件
#![feature(global_asm)]
//!
//! - `#![feature(panic_info_message)]`
//!   panic! 时，获取其中的信息并打印
#![feature(panic_info_message)]
//!
//! - `#![feature(naked_functions)]`
//!   允许使用 naked 函数，即编译器不在函数前后添加出入栈操作。
//!   这允许我们在函数中间内联汇编使用 `ret` 提前结束，而不会导致栈出现异常
#![feature(naked_functions)]
#![feature(slice_fill)]


#[macro_use]
mod console;
mod drivers;
mod fs;
mod interrupt;
mod kernel;
mod memory;
mod panic;
mod process;
mod sbi;

use crate::memory::PhysicalAddress;
use fs::*;
use process::*;
use xmas_elf::ElfFile;

extern crate alloc;

// 汇编编写的程序入口，具体见该文件
global_asm!(include_str!("entry.asm"));

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    memory::init();
    interrupt::init();
    drivers::init(dtb_pa);
    fs::init();

    // 新建一个带有内核映射的进程。需要执行的代码就在内核中
    let process = Process::new_kernel().unwrap();

    for message in 0..3 {
        let thread = Thread::new(
            process.clone(),            // 使用同一个进程
            sample_process as usize,    // 入口函数
            Some(&[message]),           // 参数
        ).unwrap();
        PROCESSOR.get().add_thread(thread, message);
    }

    // 把多余的 process 引用丢弃掉
    drop(process);

    PROCESSOR.get().run();
}

fn sample_process(message: usize) {
    loop {
        for i in 0..2000000 {
            let a = 1;
            let _b = a;
            if i % 2000000 == 0 {
                println!("thread {}", message);
            }
        }
    }
}
