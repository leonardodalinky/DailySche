//! Multilevel Feedback Queue Scheduler [`MlfqScheduler`]

use super::Scheduler;
use alloc::collections::LinkedList;

/// 将线程和调度信息打包
struct HrrnThread<ThreadType: Clone + Eq> {
    /// 进入线程池时，[`current_time`] 中的时间
    birth_time: usize,
    /// 被分配时间片的次数
    service_count: usize,
    /// 线程数据
    pub thread: ThreadType,
}

/// each level
struct MlfqLevel<ThreadType: Clone + Eq> {
    // the time interval
    // pub interval: usize,
    /// the queue
    pub queue: LinkedList<HrrnThread<ThreadType>>,
}

impl<ThreadType: Clone + Eq> MlfqLevel<ThreadType> {
    fn new() -> Self {
        Self {
            //interval: interval,
            queue: LinkedList::new(),
        }
    }
}

const MLFQ_LEVELS: usize = 5;

/// mlfq scheduler
pub struct MlfqScheduler<ThreadType: Clone + Eq> {
    /// 当前时间，单位为 `get_next()` 调用次数
    current_time: usize,
    /// 带有调度信息的线程池
    /// contains 5 levels
    levels: [MlfqLevel<ThreadType>; MLFQ_LEVELS],
}

/// `Default` 创建一个空的调度器
impl<ThreadType: Clone + Eq> Default for MlfqScheduler<ThreadType> {
    fn default() -> Self {
        Self {
            current_time: 0,
            levels: [MlfqLevel::new(), MlfqLevel::new(), MlfqLevel::new(), MlfqLevel::new(), MlfqLevel::new()],
        }
    }
}

impl<ThreadType: Clone + Eq> Scheduler<ThreadType> for MlfqScheduler<ThreadType> {
    fn add_thread<T>(&mut self, thread: ThreadType, _priority: T) {
        // decide which level would the thread join
        self.levels[0].queue.push_back(HrrnThread {
            birth_time: self.current_time,
            service_count: 0,
            thread: thread,
        });
    }
    fn get_next(&mut self) -> Option<ThreadType> {
        // 计时
        self.current_time += 1;

        for i in 0..MLFQ_LEVELS-1 {
            let level: &mut MlfqLevel<ThreadType> = &mut self.levels[i];
            if let Some(mut thread) = level.queue.pop_front() {
                thread.service_count += 1;
                let ret = Some(thread.thread.clone());
                self.levels[i + 1].queue.push_back(thread);
                return ret;
            }
        }

        // 遍历线程池，返回响应比最高者
        let current_time = self.current_time; // borrow-check
        if let Some(best) = self.levels[MLFQ_LEVELS - 1].queue.iter_mut().max_by(|x, y| {
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
        // 移除相应的线程并且确认恰移除一个线程
        let mut remove_count = 0;
        for i in 0..MLFQ_LEVELS {
            let removed = self.levels[i].queue.drain_filter(|t| t.thread == *thread);
            remove_count += removed.count();
        }
        assert_eq!(1, remove_count);
    }
    fn set_priority<T>(&mut self, _thread: ThreadType, _priority: T) {}
}
