//! 最高响应比优先算法的调度器 [`HrrnScheduler`]

use super::Scheduler;
use alloc::collections::BinaryHeap;
use core::cmp::{Reverse, min, Ordering};

/// 将线程和调度信息打包
struct StrideThread<ThreadType: Clone + Eq> {
    /// each stride in each cycle
    stride: usize,
    /// total add-up counts
    pass: usize,
    /// 线程数据
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
        let _priority = Self::get_valid_priority(_priority);
        self.pool.push(Reverse(StrideThread {
            stride: _priority,
            pass: 0,
            thread: thread
        }));
    }
    fn get_next(&mut self) -> Option<ThreadType> {
        // TODO
        // 计时
        self.current_time += 1;

        // 遍历线程池，返回响应比最高者
        let current_time = self.current_time; // borrow-check
        if let Some(best) = self.pool.iter_mut().max_by(|x, y| {
            ((current_time - x.birth_time) * y.service_count)
                .cmp(&((current_time - y.birth_time) * x.service_count))
        }) {
            best.service_count += 1;
            Some(best.thread.clone())
        } else {
            None
        }
    }
    fn remove_thread(&mut self, thread: &ThreadType) {
        // TODO:移除相应的线程并且确认恰移除一个线程
        let mut removed = self.pool.drain_filter(|t| t.thread == *thread);
        assert!(removed.next().is_some() && removed.next().is_none());
    }
    fn set_priority(&mut self, _thread: ThreadType, _priority: usize) {
        // TODO
    }

}

impl<ThreadType: Clone + Eq> StrideScheduler<ThreadType> {
    /// priority range from 0(inclusive) to 32(exclusive)
    fn get_valid_priority(p: usize) -> usize {
        min(31, p)
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