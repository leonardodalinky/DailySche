//! 最高响应比优先算法的调度器 [`HrrnScheduler`]

use super::Scheduler;
use alloc::collections::{BinaryHeap};
use core::cmp::{Reverse, min, Ordering};

/// 将线程和调度信息打包
#[derive(Clone)]
struct StrideThread<ThreadType: Clone + Eq> {
    /// each stride in each cycle
    stride: usize,
    /// total add-up counts
    pass: usize,
    /// 线程数据s
    pub thread: ThreadType,
}

/// 采用 Stride Scheduling 的调度器
pub struct StrideScheduler<ThreadType: Clone + Eq> {
    /// 带有调度信息的线程池
    pool: BinaryHeap<Reverse<StrideThread<ThreadType>>>,
}

/// `Default` 创建一个空的调度器
impl<ThreadType: Clone + Eq> Default for StrideScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            pool: BinaryHeap::new(),
        }
    }
}



impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for StrideScheduler<ThreadType> {
    fn add_thread(&mut self, thread: ThreadType, _priority: usize) {
        let _priority = Self::get_valid_stride(_priority);
        self.pool.push(Reverse(StrideThread {
            stride: _priority,
            pass: 0,
            thread: thread
        }));
    }
    fn get_next(&mut self) -> Option<ThreadType> {
        // TODO
        if let Some(t) = self.pool.pop() {
            let mut th: StrideThread<ThreadType> = t.0;
            th.pass += th.stride;
            let ret = th.thread.clone();
            self.pool.push(Reverse(th));
            Some(ret)
        }
        else {
            None
        }
    }
    fn remove_thread(&mut self, thread: &ThreadType) {
        // 移除相应的线程并且确认恰移除一个线程
        self.pool.retain(|t| t.0.thread != *thread);
    }
    fn set_priority(&mut self, thread: &ThreadType, priority: usize) {
        // set the 'stride' of specific thread
        let mut pool_vec = self.pool.clone().into_vec();
        for it in pool_vec.iter_mut() {
            if (*it).0.thread == *thread {
                (*it).0.stride = Self::get_valid_stride(priority);
            }
        }
        self.pool = BinaryHeap::from(pool_vec);
    }

}

impl<ThreadType: Clone + Eq> StrideScheduler<ThreadType> {
    /// priority range from 0(inclusive) to 32(exclusive)
    fn get_valid_stride(priority: usize) -> usize {
        33 - min(32, priority + 1)
    }
}

impl<ThreadType: Clone + Eq> PartialOrd for StrideThread<ThreadType> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pass.partial_cmp(&other.pass)
    }
}

impl<ThreadType: Clone + Eq> PartialEq for StrideThread<ThreadType> {
    fn eq(&self, other: &Self) -> bool {
        self.pass.eq(&other.pass)
    }
}

impl<ThreadType: Clone + Eq> Eq for StrideThread<ThreadType> {}

impl<ThreadType: Clone + Eq> Ord for StrideThread<ThreadType> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pass.cmp(&other.pass)
    }
}