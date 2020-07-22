# 线程

## 基本概念

**进程（Process）** 是计算机中的程序关于某数据集合上的一次运行活动，是系统进行资源分配和调度的基本单位，是操作系统结构的基础。 在当代面向线程设计的计算机结构中，进程是线程的容器。程序是指令、数据及其组织形式的描述，进程是程序的实体。是计算机中的程序关于某数据集合上的一次运行活动，是系统进行资源分配和调度的基本单位，是操作系统结构的基础。程序是指令、数据及其组织形式的描述，进程是程序的实体。

**线程（thread）** 是操作系统能够进行运算调度的最小单位。它被包含在进程之中，是进程中的实际运作单位。一条线程指的是进程中一个单一顺序的控制流，一个进程中可以并发多个线程，每条线程并行执行不同的任务。

简单来说，可以概括为如下：

* 进程：指在系统中正在运行的一个应用程序；程序一旦运行就是进程；进程——资源分配的最小单位。

* 线程：系统分配处理器时间资源的基本单元，或者说进程之内独立执行的一个单元执行流。线程——程序执行的最小单位。

在文档中，也有类似的描述：

> 出于OS对计算机系统精细管理的目的，我们通常将“正在运行”的动态特性从进程中剥离出来，这样的一个借助 CPU 和栈的执行流，我们称之为**线程 (Thread)** 。一个进程可以有多个线程，也可以如传统进程一样只有一个线程。
>
> 这样，进程虽然仍是代表一个正在运行的程序，但是其主要功能是作为**资源的分配单位**，管理页表、文件、网络等资源。而一个进程的多个线程则共享这些资源，专注于执行，从而作为**执行的调度单位**。举一个例子，为了分配给进程一段内存，我们把一整个页表交给进程，而出于某些目的（比如为了加速需要两个线程放在两个 CPU 的核上），我们需要线程的概念来进一步细化执行的方式，这时进程内部的全部这些线程看到的就是同样的页表，看到的也是相同的地址。但是需要注意的是，这些线程为了可以独立运行，有自己的栈（会放在相同地址空间的不同位置），CPU 也会以它们这些线程为一个基本调度单位。

### 线程的表示

在不同操作系统中，为每个线程所保存的信息都不同。在这里，我们提供一种基础的实现，每个线程会包括：

- **线程 ID**：用于唯一确认一个线程，它会在系统调用等时刻用到。
- **运行栈**：每个线程都必须有一个独立的运行栈，保存运行时数据。
- **线程执行上下文**：当线程不在执行时，我们需要保存其上下文（其实就是一堆**寄存器**的值），这样之后才能够将其恢复，继续运行。和之前实现的中断一样，上下文由 `Context` 类型保存。（注：这里的**线程执行上下文**与前面提到的**中断上下文**是不同的概念）
- **所属进程的记号**：同一个进程中的多个线程，会共享页表、打开文件等信息。因此，我们将它们提取出来放到线程中。
- ***内核栈***：除了线程运行必须有的运行栈，中断处理也必须有一个单独的栈。之前，我们的中断处理是直接在原来的栈上进行（我们直接将 `Context` 压入栈）。为了确保中断处理能够进行（让操作系统能够接管这样的线程），中断处理必须运行在一个准备好的、安全的栈上。这就是内核栈。不过，内核栈并没有存储在线程信息中。（注：**它的使用方法会有些复杂，我们会在后面讲解**。）

线程结构的定义，位于 `os/src/process/thread.rs`。

```rust
/// 线程的信息
pub struct Thread {
    /// 线程 ID
    pub id: ThreadID,
    /// 线程的栈
    pub stack: Range<VirtualAddress>,
    /// 线程执行上下文
    ///
    /// 当且仅当线程被暂停执行时，`context` 为 `Some`
    pub context: Mutex<Option<Context>>,
    /// 所属的进程
    pub process: Arc<RwLock<Process>>,
}
```

### 进程的表示

在我们实现的简单操作系统中，进程只需要维护页面映射，并且存储一点额外信息：

- **用户态标识**：我们会在后面进行区分内核态线程和用户态线程。
- **访存空间 `MemorySet`**：进程中的线程会共享同一个页表，即可以访问的虚拟内存空间（简称：访存空间）。

进程结构的定义，位于 `os/src/process/process.rs`。

```rust
/// 进程的信息
pub struct Process {
    /// 是否属于用户态
    pub is_user: bool,
    /// 进程中的线程公用页表 / 内存映射
    pub memory_set: MemorySet,
}
```

## 创建线程

接下来，我们的第一个目标就是创建一个线程并且让他运行起来。一个线程要开始运行，需要这些准备工作：

- 建立页表映射，需要包括以下映射空间：
  - 线程所执行的一段指令
  - 线程执行栈
  - *操作系统的部分内存空间*
- 设置起始执行的地址
- 初始化各种寄存器，比如 `sp`
- TODO：设置一些执行参数（例如 `argc` 和 `argv`等 ）

>  **思考题**：为什么线程即便与操作系统无关，也需要在内存中映射操作系统的内存空间呢？
>
> **答**：因为我们在线程中，如果需要通过中断将执行权移交至操作系统时，操作系统的部分内存需要在现在的内存空间上存在映射。

### 修改 __restore

因为启动线程需要修改各种寄存器的值，所以我们又要使用汇编了。不过，这一次我们只需要对 `interrupt.asm` 稍作修改就可以了。

在 `interrupt.asm` 中的 `__restore` 标签现在就能派上用途了。原本这段汇编代码的作用是将之前所保存的 `Context` 恢复到寄存器中，而现在我们让它使用一个精心设计的 `Context`，就可以让程序在恢复后直接进入我们的新线程。

首先我们稍作修改，添加一行 `mv sp, a0`。原本这里是读取之前存好的 `Context`，现在我们让其从 `a0` 中读取我们设计好的 `Context`。这样，我们可以直接在 Rust 代码中调用 `__restore(context)`

更改 `os/src/interrupt.asm` 中的代码。

### 线程上下文

`Context` 中至少需要下面的结构：

- 通用寄存器

  - `sp`：应当指向该线程的栈顶
  - `a0`-`a7`：按照函数调用规则，用来传递参数
  - `ra`：线程执行完应该跳转到哪里呢？在后续**系统调用**章节我们会介绍正确的处理方式。现在，我们先将其设为一个不可执行的地址，这样线程一结束就会触发页面异常

- `sepc`

  - 执行 `sret` 指令后会跳转到这里，所以 `sepc` 应当存储线程的入口地址（执行的函数地址）

- `sstatus`

  - `spp` 位按照用户态或内核态有所不同
  - `spie` 位为 1

> **`sstatus` 标志位的具体意义**
>
> - `spp`：中断前系统处于内核态（1）还是用户态（0）
> - `sie`：内核态是否允许中断。对用户态而言，无论 `sie` 取何值都开启中断
> - `spie`：中断前是否开中断（用户态中断时可能 `sie` 为 0）
>
> **硬件处理流程**
>
> - 在中断发生时，系统要切换到内核态。此时，**切换前的状态**会被保存在 **`spp`** 位中（1 表示切换前处于内核态）。同时，**切换前是否开中断**会被保存在 **`spie`** 位中，而 `sie` 位会被置 0，表示关闭中断。
> - 在中断结束，执行 `sret` 指令时，会根据 `spp` 位的值决定 `sret` 执行后是处于内核态还是用户态。与此同时，`spie` 位的值会被写入 `sie` 位，而 `spie` 位置 1。这样，特权状态和中断状态就全部恢复了。
>
> **为何如此繁琐？**
>
> - 特权状态：
>   中断处理流程必须切换到内核态，所以中断时需要用 `spp` 来保存之前的状态。
>   回忆计算机组成原理的知识，`sret` 指令必须同时完成跳转并切换状态的工作。
> - 中断状态：
>   中断刚发生时，必须关闭中断，以保证现场保存的过程不会被干扰。同理，现场恢复的过程也必须关中断。因此，需要有以上两个硬件自动执行的操作。
>   由于中断可能嵌套，在保存现场后，根据中断的种类，可能会再开启部分中断的使能。

### 进入线程

在 `os/src/process/processor.rs` 中：

```rust
/// 第一次开始运行
///
/// 从 `current_thread` 中取出 [`Context`]，然后直接调用 `interrupt.asm` 中的 `__restore`
/// 来从 `Context` 中继续执行该线程。
pub fn run(&mut self) -> ! {
    // interrupt.asm 中的标签
    extern "C" {
        fn __restore(context: usize);
    }
    /* 激活线程的页表，取得 Context。具体过程会在后面讲解 */
    unsafe {
        __restore(context);
    }
    unreachable!()
}
```

通过上面的代码，我们进入了一个线程的上下文，也就成功开启了一个线程。

线程启动后，我们也就不再需要回到 `rust_main` 函数中，之后都依赖于线程调度来切换线程。

### 启动时禁止中断

现在，我们会在线程开始运行时开启中断，而在操作系统初始化的过程中是不应该有中断的。所以，我们删去之前设置「开启中断」的代码。

代码位于 `os/interrupt/timer.rs`。

```rust
/// 初始化时钟中断
///
/// 开启时钟中断使能，并且预约第一次时钟中断
pub fn init() {
    unsafe {
        // 开启 STIE，允许时钟中断
        sie::set_stimer();
        // （删除）开启 SIE（不是 sie 寄存器），允许内核态被中断打断
        // sstatus::set_sie();
    }
    // 设置下一次时钟中断
    set_next_timeout();
}
```

## 切换线程

### 修改中断处理

在线程切换时（即时钟中断时），`handle_interrupt` 函数需要将上一个线程的 `Context` 保存起来，然后将下一个线程的 `Context` 恢复并返回。

> 注 1：为什么不直接 in-place 修改 `Context` 呢？这是因为 `handle_interrupt` 函数返回的 `Context` 指针除了存储上下文以外，还提供了内核栈的地址。这个会在后面详细阐述。
>
> 注 2：在 Rust 中，引用 `&mut` 和指针 `*mut` 只是编译器的理解不同，其本质都是一个存储对象地址的寄存器。这里返回值使用指针而不是引用，是因为其指向的位置十分特殊，其生命周期在这里没有意义。

代码位于 `os/src/interrupt/handler.rs`：

```rust
/// 中断的处理入口
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) -> *mut Context {
    /* ... */
}

/// 处理 ebreak 断点
fn breakpoint(context: &mut Context) -> *mut Context {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
    context
}

/// 处理时钟中断
fn supervisor_timer(context: &mut Context) -> *mut Context {
    timer::tick();
    PROCESSOR.get().tick(context)
}
```

可以看到，当发生断点中断时，直接返回原来的上下文（修改一下 `sepc`）；而如果是时钟中断的时候，我们通过执行 `PROCESSOR.get().tick(context)` 函数得到的返回值作为上下文，他的工作原理在下面。

### 上下文的保存和取出

在线程切换时，我们需要保存前一个线程的 `Context`，为此我们实现 `Thread::park` 函数。

代码位于 `os/src/process/thread.rs`。

```rust
/// 发生时钟中断后暂停线程，保存状态
pub fn park(&self, context: Context) {
    // 检查目前线程内的 context 应当为 None
    let mut slot = self.context.lock();
    assert!(slot.is_none());
    // 将 Context 保存到线程中
    slot.replace(context);
}
```

然后，我们需要取出下一个线程的 `Context`，为此我们实现 `Thread::run`。不过这次需要注意的是，启动一个线程除了需要 `Context`，还需要切换页表。这个操作我们也在这个方法中完成。

```rust
/// 准备执行一个线程
///
/// 激活对应进程的页表，并返回其 Context
pub fn run(&self) -> *mut Context {
    // 激活页表
    self.process.read().memory_set.activate();
    // 取出 Context
    let parked_frame = self.context.lock().take().unwrap();

    if self.process.read().is_user {
        // 用户线程则将 Context 放至内核栈顶
        KERNEL_STACK.push_context(parked_frame)
    } else {
        // 内核线程则将 Context 放至 sp 下
        let context = (parked_frame.sp() - size_of::<Context>()) as *mut Context;
        unsafe { *context = parked_frame };
        context
    }
}
```



### 线程切换

`Processor::tick` 函数的实现方式在 `os/src/process/processor.rs`。

（调度器 `scheduler` 会在后面的小节中讲解，它会选出最适合下一个被执行的线程。之后也有我个人实现的调度器。）

```rust
/// 在一个时钟中断时，替换掉 context
pub fn tick(&mut self, context: &mut Context) -> *mut Context {
    // 向调度器询问下一个线程
    if let Some(next_thread) = self.scheduler.get_next() {
        if next_thread == self.current_thread() {
            // 没有更换线程，直接返回 Context
            context
        } else {
            // 准备下一个线程
            let next_context = next_thread.run();
            let current_thread = self.current_thread.replace(next_thread).unwrap();
            // 储存当前线程 Context
            current_thread.park(*context);
            // 返回下一个线程的 Context
            next_context
        }
    } else {
        panic!("all threads terminated, shutting down");
    }
}
```

**思考**：在 `run` 函数中，我们在一开始就激活了页表，会不会导致后续流程无法正常执行？

**答**：不会。因为每个内存映射中，都存在对内核部分的映射，而 `run` 函数中的 `pc` 正好处于内核段中。

## 内核栈

### 为什么 / 怎么做

在实现内核栈之前，让我们先检查一下需求和我们的解决办法。

- **不是每个线程都需要一个独立的内核栈**，因为内核栈只会在中断时使用，而中断结束后就不再使用。在只有一个 CPU 的情况下，不会有两个线程同时出现中断，**所以我们只需要实现一个共用的内核栈就可以了**。
- **每个线程都需要能够在中断时第一时间找到内核栈的地址**。这时，所有通用寄存器的值都无法预知，也无法从某个变量来加载地址。为此，**我们将内核栈的地址存放到内核态使用的特权寄存器 `sscratch` 中**。这个寄存器只能在内核态访问，这样在中断发生时，就可以安全地找到内核栈了。

因此，我们的做法就是：

- 预留一段空间作为内核栈
- 运行线程时，在 `sscratch` 寄存器中保存内核栈指针
- 如果线程遇到中断，则从将 `Context` 压入 `sscratch` 指向的栈中（`Context` 的地址为 `sscratch - size_of::()`），同时用新的栈地址来替换 `sp`（此时 `sp` 也会被复制到 `a0` 作为 `handle_interrupt` 的参数）
- 从中断中返回时（`__restore` 时），`a0` 应指向**被压在内核栈中的 `Context`**。此时出栈 `Context` 并且将栈顶保存到 `sscratch` 中

#### 内核栈定义

我们直接使用一个 `static mut` 来指定一段空间作为栈。代码位于 `os/src/process/kernel_stack.rs`。

```rust
/// 内核栈
#[repr(align(16))]
#[repr(C)]
pub struct KernelStack([u8; KERNEL_STACK_SIZE]);

/// 公用的内核栈
pub static mut KERNEL_STACK: KernelStack = KernelStack([0; STACK_SIZE]);
```

在我们创建线程时，需要使用的操作就是在**内核栈顶**压入一个初始状态 `Context`：

```rust
impl KernelStack {
    /// 在栈顶加入 Context 并且返回新的栈顶指针
    pub fn push_context(&mut self, context: Context) -> *mut Context {
        // 栈顶
        let stack_top = &self.0 as *const _ as usize + size_of::<Self>();
        // Context 的位置
        let push_address = (stack_top - size_of::<Context>()) as *mut Context;
        unsafe {
            *push_address = context;
        }
        push_address
    }
}
```

#### 修改中断汇编

在这个汇编代码中，我们需要加入对 `sscratch` 的判断和使用。修改 `os/src/interrupt.asm`。

```assembly
__interrupt:
    # 因为线程当前的栈不一定可用，必须切换到内核栈来保存 Context 并进行中断流程
    # 因此，我们使用 sscratch 寄存器保存内核栈地址
    # 思考：sscratch 的值最初是在什么地方写入的？

    # 交换 sp 和 sscratch（切换到内核栈）
    csrrw   sp, sscratch, sp
    # 在内核栈开辟 Context 的空间
    addi    sp, sp, -36*8

    # 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    # 将本来的栈地址 sp（即 x2）保存
    csrr    x1, sscratch
    SAVE    x1, 2
    SAVE    x3, 3
    SAVE    x4, 4

    # ...
```

以及事后的恢复：

```assembly
# 离开中断
# 此时内核栈顶被推入了一个 Context，而 a0 指向它
# 接下来从 Context 中恢复所有寄存器，并将 Context 出栈（用 sscratch 记录内核栈地址）
# 最后跳转至恢复的 sepc 的位置
__restore:
    # 从 a0 中读取 sp
    # 思考：a0 是在哪里被赋值的？（有两种情况）
    mv      sp, a0
    # 恢复 CSR
    LOAD    t0, 32
    LOAD    t1, 33
    csrw    sstatus, t0
    csrw    sepc, t1
    # 将内核栈地址写入 sscratch
    addi    t0, sp, 36*8
    csrw    sscratch, t0

    # 恢复通用寄存器
    # ...
```

## 调度器

### 处理器抽象

我们已经可以创建和保存线程了，现在，我们再抽象出「处理器」来存放和管理线程池。同时，也需要存放和管理目前正在执行的线程（即中断前执行的线程，因为操作系统在工作时是处于中断、异常或系统调用服务之中）。

代码位于 `os/src/process/processor.rs`。

```rust
lazy_static! {
    /// 全局的 [`Processor`]
    pub static ref PROCESSOR: UnsafeWrapper<Processor> = Default::default();
}

/// 线程调度和管理
#[derive(Default)]
pub struct Processor {
    /// 当前正在执行的线程
    current_thread: Option<Arc<Thread>>,
    /// 线程调度器，记录所有线程
    scheduler: SchedulerImpl<Arc<Thread>>,
}
```

注意到这里我们用了一个 `UnsafeWrapper`，这个东西相当于 Rust 提供的 `UnsafeCell`，或者 C 语言的指针：任何线程都可以随时从中获取一个 `&'static mut` 引用。由于在我们的设计中，**只有时钟中断（以及异常或未来的系统调用）时可以使用 `PROCESSOR`**，而在此过程中，操作系统是关闭时钟中断的。因此，这里使用 `UnsafeCell` 是安全的。

### 调度器

调度器的算法有许多种，我们将它提取出一个 trait 作为接口。代码位于 `os/src/algorithm/src/scheduler/mod.rs`。

```rust
/// 线程调度器
///
/// 这里 `ThreadType` 就是 `Arc<Thread>`
pub trait Scheduler<ThreadType: Clone + Eq>: Default {
    /// 向线程池中添加一个线程
    fn add_thread<T>(&mut self, thread: ThreadType, priority: T);
    /// 获取下一个时间段应当执行的线程
    fn get_next(&mut self) -> Option<ThreadType>;
    /// 移除一个线程
    fn remove_thread(&mut self, thread: ThreadType);
    /// 设置线程的优先级
    fn set_priority<T>(&mut self, thread: ThreadType, priority: T);
}
```

具体的算法就不在此展开了，我们可以参照目录 `os/src/algorithm/src/scheduler` 下的一些样例。

在同目录下的 `scheduler.md` 文件中，也有个人关于调度器的理解。

### 运行！

最后，让我们补充 `Processor::run` 的实现，让我们运行起第一个线程！

代码位于 `os/src/process/processor.rs`。

```rust
/// 第一次开始运行
pub fn run(&mut self) -> ! {
    // interrupt.asm 中的标签
    extern "C" {
        fn __restore(context: usize);
    }
    // 从 current_thread 中取出 Context
    let context = self.current_thread().run();
    // 从此将没有回头
    unsafe {
        __restore(context as usize);
    }
    unreachable!()
}
```

修改 `main.rs`，我们就可以跑起来多线程了。

```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    memory::init();
    interrupt::init();

    // 新建一个带有内核映射的进程。需要执行的代码就在内核中
    let process = Process::new_kernel().unwrap();

    for message in 0..8 {
        let thread = Thread::new(
            process.clone(),            // 使用同一个进程
            sample_process as usize,    // 入口函数
            Some(&[message]),           // 参数
        ).unwrap();
        PROCESSOR.get().add_thread(thread);
    }

    // 把多余的 process 引用丢弃掉
    drop(process);

    PROCESSOR.get().run();
}

fn sample_process(message: usize) {
    for i in 0..1000000 {
        if i % 200000 == 0 {
            println!("thread {}", message);
        }
    }
}
```

上面代码输出结果如下：

```
thread 7
thread 6
thread 5
...
thread 7
...
thread 2
thread 1
thread 0
...
```

## 实验题（上）

1. **原理**：线程切换之中，页表是何时切换的？页表的切换会不会影响程序 / 操作系统的运行？为什么？

   **答**：页表是在时钟中断发生后， `Process::prepare_next_thread()` 中换入了新线程的页表。它不会影响操作系统，因为每个页表上都存在对操作系统的映射。

2. 设计：如果不使用 `sscratch` 提供内核栈，而是像原来一样，遇到中断就直接将上下文压栈，请举出（思路即可，无需代码）：

   - 一种情况不会出现问题：在线程栈大小足够的情况下，发生 `breakpoint` 等对线程执行无影响的函数。
   - 一种情况导致异常无法处理（指无法进入 `handle_interrupt`）：当栈指针 `sp` 到达一个不可读写的区域，比如爆栈时，可能会产生此种情况。
   - 一种情况导致产生嵌套异常（指第二个异常能够进行到调用 `handle_interrupt`，不考虑后续执行情况）：*运行两个线程。在两个线程切换的时候，会需要切换页表。但是此时操作系统运行在前一个线程的栈上，一旦切换，再访问栈就会导致缺页，因为每个线程的栈只在自己的页表中。*这里直接抄写了答案，因为我对“嵌套异常”这个概念感到奇怪。此处的答案是会发生异常的嵌套，而事实上我们的内核目前无法处理异常的嵌套（即异常中触发异常），所以产生运行时问题。
   - 一种情况导致一个用户进程（先不考虑是怎么来的）可以将自己变为内核进程，或以内核态执行自己的代码：当用户栈爆栈的时候，可能修改内核中的信息。

3. **实验**：当键盘按下 Ctrl + C 时，操作系统应该能够捕捉到中断。实现操作系统捕获该信号并结束当前运行的线程（你可能需要阅读一点在实验指导中没有提到的代码）

   **思路**：在 `driver` 和 `fs` 模块都成功加载后，修改 `handler::interrupt::supervisor_external` 函数如下：

   ```rust
   /// 处理外部中断，只实现了键盘输入
   fn supervisor_external(context: &mut Context) -> *mut Context {
       let mut c = console_getchar();
       // ctrl-c
       // shutdown current thread
       if c == 3 {
           PROCESSOR.get().kill_current_thread();
           println!("Kill current Thread!");
           return PROCESSOR.get().prepare_next_thread();
       }
       if c <= 255 {
           if c == '\r' as usize {
               c = '\n' as usize;
           }
           STDIN.push(c as u8);
       }
       context
   }
   ```

   其中 `if c == 3` 中的 3 表示 `ctrl-c` 的 `ascii` 码。

   程序运行过程中按下 `ctrl-c` 后，即会终止当前线程，并切换到下一个线程。

   代码位于 `os/interrupt/handler.rs` 中。

4. **实验**：实现线程的 `fork()`。目前的内核线程不能进行系统调用，所以我们先简化地实现为“按 F 进入 fork”。fork 后应当为目前的线程复制一份几乎一样的拷贝，新线程与旧线程同属一个进程，公用页表和大部分内存空间，而新线程的栈是一份拷贝。

   **思路**：模仿线程的 `new` 的过程，获得一个新的 `fork_with_context` 函数

   ```rust
       /// fork current thread
       pub fn fork_with_context(&self, context: Option<Context>) -> Arc<Thread> {
           let stack: Range<VirtualAddress> = self.process
           .write()
           .alloc_page_range(STACK_SIZE, Flags::READABLE | Flags::WRITABLE).expect("failed to fork stack");
   
           // refresh page tables
           unsafe {
               asm!("sfence.vma");
           }
           // copy the stack content
           // the kernel is running now
           unsafe {
               let src = self.stack.start.0 as *mut usize;
               let dst = stack.start.0 as *mut usize;
               core::ptr::copy_nonoverlapping(
                   src, 
                   dst, 
                   STACK_SIZE / core::mem::size_of::<usize>()
               );
           }
   
           let mut context_unwrap = context.expect("fork context is none");
           context_unwrap.set_sp(context_unwrap.sp() - usize::from(self.stack.start) +
               usize::from(stack.start));
           Arc::new(Thread {
               id: unsafe {
                   THREAD_COUNTER += 1;
                   THREAD_COUNTER
               },
               stack: stack,
               process: self.process.clone(),
               inner: Mutex::new(ThreadInner {
                   context: Some(context_unwrap),
                   sleeping: self.inner().sleeping,
                   descriptors: vec![STDIN.clone(), STDOUT.clone()],
               }),
           })
       }
   ```

   这个函数复制当前的线程除了 `Context` 的部分，并且重新开了一个函数栈，同时使用 `core::ptr::copy_nonoverlapping` 函数，将栈的内容完整的复制过去，同时将新的栈指针指向正确的位置。

   代码位于 `os/src/process/thread.rs` 中。

   这种 fork 有几个问题：

   1. 由于这属于线程的 fork，如果在线程中拥有指针，则在 fork 后指针会失效，十分危险。

   2. 当线程死亡后，由于 `segment` 中并没有删除死亡线程的运行栈段，所以会导致下一次 fork 的时候，虚拟内存的地址会不断升高。

## 实验题（下）

1. **实验**：了解并实现 Stride Scheduling 调度算法，为不同线程设置不同优先级，使得其获得与优先级成正比的运行时间。

   **答**：代码位于 `os/src/algorithm/src/scheduler/stride_scheduler.rs` 。

   Stride Scheduling 算法简要概括如下：线程拥有一个 pass 属性和 stride 属性。每当线程被选用，则其 pass 属性就自增 stride 的值。每次选用 pass 值最小的线程。

   为了方便起见，优先度的取值范围为 $[0,31]$，优先度与运行时间成正比。

2. **分析**：

   - 在 Stride Scheduling 算法下，如果一个线程进入了一段时间的等待（例如等待输入，此时它不会被运行），会发生什么？
   
     **答**：线程进入休眠状态后，当其他线程运行一段时间后，当切回该线程时，由于此线程的 pass 值相对于其他线程小很多，所以其将长时间抢占执行权。
   
   - 对于两个优先级分别为 9 和 1 的线程，连续 10 个时间片中，前者的运行次数一定更多吗？
   
     **答**：不一定。如果低优先级的线程在中途才加入，由于其积累的 pass 值较小，所以也会在一定时间内抢占大部分执行权。
   
   - 你认为 Stride Scheduling 算法有什么不合理之处？可以怎样改进？
   
     **答**：针对上面提到的由于 pass 值积累所导致的长时间抢占执行权的问题，可以在线程重新加入（或被唤醒）时，将其 pass 值设定为当前活动线程中最小的 pass 值，以重新公平竞争。