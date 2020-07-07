//! 中断模块
//! 
//! 

pub mod handler;
pub mod context;
pub mod timer;

/// 初始化中断相关的子模块
/// 
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    println!("mod interrupt initialized");
}