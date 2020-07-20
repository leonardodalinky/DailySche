# 调度器

## 调度器解读

在开始实现自己的调度器之前，我们先分析一下 Tutorial 中的调度器实现。

在我们的 Tutorial 中，一共实现了两种调度器：

* 先进先出的调度器（FIFO，First In First Out）
* 基于最高响应比的调度器（HRRN，Highest Response Ratio Next）

代码都位于 `os/src/process` 下。

### FIFO调度器

FIFO 调度器的实现相对比较简单。根据源代码来看，其基本原理为：维护一个队列，存储着现在运行中的线程。当外部索要下一个可供执行的线程时，就返回队列中的第一个线程，同时将其放到队列末尾。

### HRRN调度器

HRRN 调度器的工作原理如下：

当外部索要下一个可供执行的线程时，通过计算所有待选线程的响应比，然后挑选出目前响应比最高的线程。

响应比的计算方式为：
$$
响应比=\frac{等待时间+要求服务时间}{要求服务时间}
$$
在 Tutorial 的源码中，由于每一次的要求服务时间都为一个时间片，所以响应比的方式略有不同，但是基本思路都跟随着上述的公式。

## 基于HRRN的多级反馈队列调度器

### 原理

在上面两个调度器的基础下，我个人实现了一个简单的多级反馈队列调度器。

多级反馈队列的调度器的基本原理如下（参考[百度百科](https://baike.baidu.com/item/多级反馈队列调度算法)）：

1. 进程在进入待调度的队列等待时，首先进入优先级最高的Q1等待。
2. 首先调度优先级高的队列中的进程。若高优先级中队列中已没有调度的进程，则调度次优先级队列中的进程。例如：Q1,Q2,Q3 三个队列，当且仅当在 Q1 中没有进程等待时才去调度 Q2 ，同理，只有 Q1,Q2 都为空时才会去调度 Q3。
3. 对于同一个队列中的各个进程，按照 FCFS 分配时间片调度。比如 Q1 队列的时间片为 N，那么 Q1 中的作业在经历了 N 个时间片后若还没有完成，则进入 Q2 队列等待，若 Q2 的时间片用完后作业还不能完成，一直进入下一级队列，直至完成。
4. 在最后一个队列QN中的各个进程，按照时间片轮转分配时间片调度。
5. 在低优先级的队列中的进程在运行时，又有新到达的作业，此时须立即把正在运行的进程放回当前队列的队尾，然后把处理机分给高优先级进程。换而言之，任何时刻，只有当第 1~(i-1) ​队列全部为空时，才会去执行第i队列的进程（抢占式）。特别说明，当再度运行到当前队列的该进程时，仅分配上次还未完成的时间片，不再分配该队列对应的完整时间片。

基本原理如上。

但是在实施时，发现了一个严重的问题，并作出一些调整。

我们的 Tutorial 中的调度器接口，并没有提供更改时间片大小的接口，即无法调整时间中断间隔。为了兼容我们上面两个已有的调度器的接口，我对多级反馈队列的原理做出了修正。

修正如下：

1. 每个队列中的时间片大小相同。此为无奈之举。
2. 优先度最低的队列中，使用HRRN调度，而不是简单的时间片轮转。
3. Priority 的类型由于属于模板参数类型，为了兼容接口，故无法将其转换为 usize 类型。因此，在此放弃对于 Priority 的设置。考虑之后更改接口以适配。
4. 共建立 5 个队列。

### 部分代码实现

```rust
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

        // 遍历非最底层的队列
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
```

