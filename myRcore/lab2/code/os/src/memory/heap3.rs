//! 自己实现动态分配器，使用此文件替换 heap.rs
//!
//! 具体分配算法在 algorithm::allocator 里面实现，
//! 这里将其中的 RAllocator 接入 GlobalAlloc，作为全局分配器
//! Base on heap2.rs

use super::config::KERNEL_HEAP_SIZE;
use algorithm::{RAllocator, RAllocatorImpl};
use core::cell::UnsafeCell;

/// 进行动态内存分配所用的堆空间
///
/// 大小为 [`KERNEL_HEAP_SIZE`]
/// 这段空间编译后会被放在操作系统执行程序的 bss 段
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// Use a lock to ensure safety
#[global_allocator]
static mut HEAP: Heap = Heap(UnsafeCell::new(None));

/// Heap 将分配器封装并放在 static 中。
struct Heap(UnsafeCell<Option<RAllocatorImpl>>);

/// 利用 VectorAllocator 的接口实现全局分配器的 GlobalAlloc trait
unsafe impl alloc::alloc::GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = (*self.0.get())
            .as_mut()
            .expect("Heap not initilize.")
            .alloc(layout.size(), layout.align())
            .expect("Heap overflow");
        return ptr as *mut u8;
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        (*self.0.get())
            .as_mut()
            .expect("Heap not initilize.")
            .dealloc(ptr as usize, layout.size(), layout.align());
    }
}

unsafe impl Sync for Heap {}

/// 初始化操作系统运行时堆空间
pub fn init() {
    // 告诉分配器使用这一段预留的空间作为堆
    hello();
    unsafe {
        let t = RAllocatorImpl::new(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
        (*HEAP.0.get()).replace(t);
    }
}

pub fn hello() {
    let _a = 1;
}

/// 空间分配错误的回调，直接 panic 退出
#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
