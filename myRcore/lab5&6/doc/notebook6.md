## 构建用户程序框架

接下来我们要做的工作，和实验准备中为操作系统「去除依赖」的工作十分类似：我们需要为用户程序提供一个类似的没有Rust std标准运行时依赖的极简运行时环境。这里我们会快速梳理一遍我们为用户程序进行的流程。

### 建立 crate

我们在与 `os` 的旁边建立一个 `user` crate。此时，我们移除默认的 `main.rs`，而是在 `src` 目录下建立 `lib` 和 `bin` 子目录， 在 `lib` 中存放的是极简运行时环境，在 `bin` 中存放的源文件会被编译成多个单独的执行文件。

运行命令

```bash
cargo new --bin user
```

目录结构

```
rCore-Tutorial
  - os
  - user
    - src
      - bin
        - hello_world.rs
      - lib.rs
    - Cargo.toml
```

### 基础框架搭建

和操作系统一样，我们需要为用户程序移除 std 依赖，并且补充一些必要的功能。

#### `lib.rs`

- `#![no_std]` 移除标准库
- `#![feature(...)]` 开启一些不稳定的功能
- `#[global_allocator]` 使用库来实现动态内存分配
- `#[panic_handler]` panic 时终止

#### 其他文件

- `.cargo/config` 设置编译目标为 RISC-V 64
- `console.rs` 实现 `print!` `println!` 宏

## 打包为磁盘镜像

在上一章我们已经实现了文件系统，并且可以让操作系统加载磁盘镜像。现在，我们只需要利用工具将编译后的用户程序打包为镜像，就可以使用了。

### 工具安装

通过 cargo 来安装 `rcore-fs-fuse` 工具：

运行命令

```bash
cargo install rcore-fs-fuse --git https://github.com/rcore-os/rcore-fs
```

### 打包

这个工具可以将一个目录打包成 SimpleFileSystem 格式的磁盘镜像。为此，我们需要将编译得到的 ELF 文件单独放在一个导出目录中，即 `user/build/disk`。

位于 `user/Makefile`

```makefile
build: dependency
    # 编译
    @cargo build
    @echo Targets: $(patsubst $(SRC_DIR)/%.rs, %, $(SRC_FILES))
    # 移除原有的所有文件
    @rm -rf $(OUT_DIR)
    @mkdir -p $(OUT_DIR)
    # 复制编译生成的 ELF 至目标目录
    @cp $(BIN_FILES) $(OUT_DIR)
    # 使用 rcore-fs-fuse 工具进行打包
    @rcore-fs-fuse --fs sfs $(IMG_FILE) $(OUT_DIR) zip
    # 将镜像文件的格式转换为 QEMU 使用的高级格式
    @qemu-img convert -f raw $(IMG_FILE) -O qcow2 $(QCOW_FILE)
    # 提升镜像文件的容量（并非实际大小），来允许更多数据写入
    @qemu-img resize $(QCOW_FILE) +1G
```

在 `os/Makefile` 中指定我们新生成的 `QCOW_FILE` 为加载镜像，就可以在操作系统中看到打包好的目录了。

## 解析 ELF 文件并创建线程

在之前实现内核线程时，我们只需要为线程指定一个起始位置就够了，因为所有的代码都在操作系统之中。但是现在，我们需要从 ELF 文件中加载用户程序的代码和数据信息，并且映射到内存中。

当然，我们不需要自己实现 ELF 文件解析器，因为有 `xmas-elf` 这个 crate 替我们实现了 ELF 的解析。

### `xmas-elf` 解析器

tips：如果 IDE 无法对其中的类型进行推断，可以在 rustdoc 中找到该 crate 进行查阅。

#### 读取文件内容

`xmas-elf` 需要将 ELF 文件首先读取到内存中。在上一章文件系统的基础上，我们很容易为 `INode` 添加一个将整个文件作为 `[u8]` 读取出来的方法：

位于 `os/src/fs/inode_ext.rs`

```rust
fn readall(&self) -> Result<Vec<u8>> {
    // 从文件头读取长度
    let size = self.metadata()?.size;
    // 构建 Vec 并读取
    let mut buffer = Vec::with_capacity(size);
    unsafe { buffer.set_len(size) };
    self.read_at(0, buffer.as_mut_slice())?;
    Ok(buffer)
}
```

### 解析各个字段

对于 ELF 中的不同字段，其存放的地址通常是不连续的，同时其权限也会有所不同。我们利用 `xmas-elf` 库中的接口，便可以从读出的 ELF 文件中对应建立 `MemorySet`。

注意到，用户程序也会首先映射所有内核态的空间，否则将无法进行中断处理。

位于 `os/src/memory/mapping/memory_set.rs`

```rust
/// 通过 elf 文件创建内存映射（不包括栈）
pub fn from_elf(file: &ElfFile, is_user: bool) -> MemoryResult<MemorySet> {
    // 建立带有内核映射的 MemorySet
    let mut memory_set = MemorySet::new_kernel()?;

    // 遍历 elf 文件的所有部分
    for program_header in file.program_iter() {
        if program_header.get_type() != Ok(Type::Load) {
            continue;
        }
        // 从每个字段读取「起始地址」「大小」和「数据」
        let start = VirtualAddress(program_header.virtual_addr() as usize);
        let size = program_header.mem_size() as usize;
        let data: &[u8] =
            if let SegmentData::Undefined(data) = program_header.get_data(file).unwrap() 			 {
                data
            } else {
                return Err("unsupported elf format");
            };

        // 将每一部分作为 Segment 进行映射
        let segment = Segment {
            map_type: MapType::Framed,
            range: Range::from(start..(start + size)),
            flags: Flags::user(is_user)
                | Flags::readable(program_header.flags().is_read())
                | Flags::writable(program_header.flags().is_write())
                | Flags::executable(program_header.flags().is_execute()),
        };

        // 建立映射并复制数据
        memory_set.add_segment(segment, Some(data))?;
    }

    Ok(memory_set)
}
```

### 加载数据到内存中

思考：我们在为用户程序建立映射时，虚拟地址是 ELF 文件中写明的，那物理地址是程序在磁盘中存储的地址吗？这样做有什么问题吗？

> 我们在模拟器上运行可能不觉得，但是如果直接映射磁盘空间，使用时会带来巨大的延迟，所以需要在程序准备运行时，将其磁盘中的数据复制到内存中。如果程序较大，操作系统可能只会复制少量数据，而更多的则在需要时再加载。当然，我们实现的简单操作系统就一次性全都加载到内存中了。
>
> 而且，就算是想要直接映射磁盘空间，也不一定可行。这是因为虚实地址转换时，页内偏移是不变的。这是就无法保证在 ELF 中指定的地址和其在磁盘中的地址满足这样的关系。

我们将修改 `Mapping::map` 函数，为其增加一个参数表示用于初始化的数据。在实现时，有一些重要的细节需要考虑。

- 因为用户程序的内存分配是动态的，其分配到的物理页面不一定连续，所以必须单独考虑每一个页面
- 每一个字段的长度不一定是页大小的倍数，所以需要考虑不足一个页时的复制情况
- 程序有一个 bss 段，它在 ELF 中不保存数据，而其在加载到内存是需要零初始化
- 对于一个页面，有其**物理地址**、**虚拟地址**和**待加载数据的地址**。此时，是不是直接从**待加载数据的地址**拷贝到页面的**虚拟地址**，如同 `memcpy` 一样就可以呢？

> 在目前的框架中，只有当线程将要运行时，才会加载其页表。因此，除非我们额外的在每映射一个页面之后，就更新一次页表并且刷新 TLB，否则此时的**虚拟地址**是无法访问的。
>
> 但是，我们通过分配器得到了页面的**物理地址**，而这个物理地址实际上已经在内核的线性映射当中了。所以，这里实际上用的是**物理地址**来写入数据

具体的实现，可以查看 `os/src/memory/mapping/mapping.rs` 中的 `Mapping::map` 函数。

### 运行 Hello World？

现在，我们就可以在操作系统中运行磁盘镜像中的用户程序了，代码示例如下：

位于 `os/src/main.rs`

```rust
// 从文件系统中找到程序
let app = fs::ROOT_INODE.find("hello_world").unwrap();
// 读取数据
let data = app.readall().unwrap();
// 解析 ELF 文件
let elf = ElfFile::new(data.as_slice()).unwrap();
// 利用 ELF 文件创建线程，映射空间并加载数据
let process = Process::from_elf(&elf, true).unwrap();
// 再从 ELF 中读出程序入口地址
let thread = Thread::new(process, elf.header.pt2.entry_point() as usize, None).unwrap();
// 添加线程
PROCESSOR.lock().add_thread(thread);
```

可惜的是，我们不能像内核线程一样在用户程序中直接使用 `print`。前者是基于 OpenSBI 的机器态 SBI 调用，而为了让用户程序能够打印字符，我们还需要在操作系统中实现系统调用来给用户进程提供服务。

## 实现系统调用

目前，我们实现 `sys_read` `sys_write` 和 `sys_exit` 三个简单的系统调用。通过学习它们的实现，更多的系统调用也并没有多难。

### 用户程序中调用系统调用

在用户程序中实现系统调用比较容易，就像我们之前在操作系统中使用 `sbi_call` 一样，只需要符合规则传递参数即可。而且这一次我们甚至不需要参考任何标准，每个人都可以为自己的操作系统实现自己的标准。

例如，在实验指导中，系统调用的编号使用了 musl 中的编码和参数格式。但实际上，在实现操作系统的时候，编码和参数格式都可以随意调整，只要在用户程序中的调用和操作系统中的解释相符即可。

代码示例

```rust
// musl 中的 sys_read 调用格式
llvm_asm!("ecall" :
    "={x10}" (/* 返回读取长度 */) :
    "{x10}" (/* 文件描述符 */),
    "{x11}" (/* 读取缓冲区 */),
    "{x12}" (/* 缓冲区长度 */),
    "{x17}" (/* sys_read 编号 63 */) ::
);
// 一种可能的 sys_read 调用格式
llvm_asm!("ecall" :
    "={x10}" (/* 现在的时间 */),
    "={x11}" (/* 今天的天气 */),
    "={x12}" (/* 读取一个字符 */) :
    "{x20}" (/* sys_read 编号 0x595_7ead */) ::
);
```

实验指导提供了第一种无趣的系统调用格式。

### 避免忙等待

在常见操作系统中，一些延迟非常大的操作，例如文件读写、网络通讯，都可以使用异步接口来进行。但是为了实现更加简便，我们的读写系统调用都是阻塞的。在 `sys_read` 中，使用了 `loop` 来保证仅当成功读取字符时才返回。

此时，如果用户程序需要获取从控制台输入的字符，但是此时并没有任何字符到来。那么，程序将被阻塞，而操作系统的职责就是尽量减少线程执行无用阻塞占用 CPU 的时间，而是将这段时间分配给其他可以执行的线程。具体的做法，将会在后面**条件变量**的章节讲述。

### 操作系统中实现系统调用

在操作系统中，系统调用的实现和中断处理一样，有同样的入口，而针对不同的参数设置不同的处理流程。为了简化流程，我们不妨把系统调用的处理结果分为三类：

- 返回一个数值，程序继续执行
- 程序进入等待
- 程序将被终止

#### 系统调用的处理流程

- 首先，从相应的寄存器中取出调用代号和参数
- 根据调用代号，进入不同的处理流程，得到处理结果
  - 返回数值并继续执行：
    - 返回值存放在 `x10` 寄存器，`sepc += 4`，继续此 `context` 的执行
  - 程序进入等待
    - 同样需要更新 `x10` 和 `sepc`，但是需要将当前线程标记为等待，切换其他线程来执行
  - 程序终止
    - 不需要考虑系统调用的返回，直接删除线程

#### 具体的调用实现

那么具体该如何实现读 / 写系统调用呢？这里我们会利用文件的统一接口 `INode`，使用其中的 `read_at()` 和 `write_at()` 接口即可。下一节就将讲解如何处理文件描述符。

## 处理文件描述符

尽管很不像，但是在大多操作系统中，标准输入输出流 `stdin` 和 `stdout` 虽然叫做「流」，但它们都有文件的接口。我们同样也会将它们实现成为文件。

但是不用担心，作为文件的许多功能，`stdin` 和 `stdout` 都不会支持。我们只需要为其实现最简单的读写接口。

### 进程打开的文件

操作系统需要为进程维护一个进程打开的文件清单。其中，一定存在的是 `stdin` `stdout` 和 `stderr`。为了简便，我们只实现 `stdin` 和 `stdout`，它们的文件描述符数值分别为 0 和 1。

### `stdout`

输出流最为简单：每当遇到系统调用时，直接将缓冲区中的字符通过 SBI 调用打印出去。

### `stdin`

输入流较为复杂：每当遇到系统调用时，通过中断或轮询方式获取字符：如果有，就进一步获取；如果没有就等待。直到收到约定长度的字符串才返回。

#### 外部中断

对于用户程序而言，外部输入是随时主动读取的数据。但是事实上外部输入通常时间短暂且不会等待，需要操作系统立即处理并缓冲下来，再等待程序进行读取。所以，每一个键盘按键对于操作系统而言都是一次短暂的中断。

而在之前的实验中操作系统不会因为一个按键就崩溃，是因为 OpenSBI 默认会关闭各种外部中断。但是现在我们需要将其打开，来接受按键信息。

位于 `os/src/interrupt/handler.rs`

```rust
/// 初始化中断处理
///
/// 把中断入口 `__interrupt` 写入 `stvec` 中，并且开启中断使能
pub fn init() {
    unsafe {
        extern "C" {
            /// `interrupt.asm` 中的中断入口
            fn __interrupt();
        }
        // 使用 Direct 模式，将中断入口设置为 `__interrupt`
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);

        // 开启外部中断使能
        sie::set_sext();

        // 在 OpenSBI 中开启外部中断
        *PhysicalAddress(0x0c00_2080).deref_kernel() = 1 << 10;
        // 在 OpenSBI 中开启串口
        *PhysicalAddress(0x1000_0004).deref_kernel() = 0x0bu8;
        *PhysicalAddress(0x1000_0001).deref_kernel() = 0x01u8;
    }
}
```

这里，我们需要按照 OpenSBI 的接口在指定的地址进行配置。好在这些地址都在文件系统映射的空间内，就不需要再为其单独建立内存映射了。开启中断使能后，任何一个按键都会导致程序进入 `unimplemented!` 的区域。

#### 实现输入流

输入流则需要配有一个缓冲区，我们可以用 `alloc::collections::VecDeque` 来实现。在遇到键盘中断时，调用 `sbi_call` 来获取字符并加入到缓冲区中。当遇到系统调用 `sys_read` 时，再相应从缓冲区中取出一定数量的字符。

那么，如果遇到了 `sys_read` 系统调用，而缓冲区并没有数据可以读取，应该如何让线程进行等待，而又不浪费 CPU 资源呢？

## 条件变量

条件变量（conditional variable）的常见接口是这样的：

- wait：当前线程开始等待这个条件变量
- notify_one：让某一个等待此条件变量的线程继续运行
- notify_all：让所有等待此变量的线程继续运行

条件变量和互斥锁的区别在于，互斥锁解铃还须系铃人，但条件变量可以由任何来源发出 notify 信号。同时，互斥锁的一次 lock 一定对应一次 unlock，但条件变量多次 notify 只能保证 wait 的线程执行次数不超过 notify 次数。

为输入流加入条件变量后，就可以使得调用 `sys_read` 的线程在等待期间保持休眠，不被调度器选中，消耗 CPU 资源。

### 调整调度器

为了继续沿用调度算法，不带来太多修改，我们为线程池单独设立一个「休眠区」，其中保存的线程与调度器互斥。当线程进入等待，就将它从调度器中取出，避免之后再被无用唤起。

位于 `os/src/process/processor.rs`

```rust
pub struct Processor {
    /// 当前正在执行的线程
    current_thread: Option<Arc<Thread>>,
    /// 线程调度器，记录活跃线程
    scheduler: SchedulerImpl<Arc<Thread>>,
    /// 保存休眠线程
    sleeping_threads: HashSet<Arc<Thread>>,
}
```

### 实现条件变量

条件变量会被包含在输入流等涉及等待和唤起的结构中，而一个条件变量保存的就是所有等待它的线程。

位于 `os/src/kernel/condvar.rs`

```rust
#[derive(Default)]
pub struct Condvar {
    /// 所有等待此条件变量的线程
    watchers: Mutex<VecDeque<Arc<Thread>>>,
}
```

当一个线程调用 `sys_read` 而缓冲区为空时，就会将其加入条件变量的 `watcher` 中，同时在 `Processor` 中移出活跃线程。而当键盘中断到来，读取到字符时，就会将线程重新放回调度器中，准备下一次调用。

**开放思考**：如果多个线程同时等待输入流会怎么样？有什么解决方案吗？

**答**：我认为，可以在每个进程下设置输入缓存区，当进程中有线程等待输入时，将输入流复制一份到进程中，而线程从缓存区中读取字符。

## 实验题

1. **原理**：使用条件变量之后，分别从线程和操作系统的角度而言读取字符的系统调用是阻塞的还是非阻塞的？

   **答**：对于线程而言，读取字符是阻塞的过程，线程会等待字符被读取。而对于操作系统而言，等待读取的线程会被置出线程调度队列，转而执行其他的线程，所以是非阻塞的。

2. **设计**：如果要让用户线程能够使用 `Vec` 等，需要做哪些工作？如果要让用户线程能够使用大于其栈大小的动态分配空间，需要做哪些工作？

   **答**：若要使用户现场可使用 `Vec` 等动态数据结构，需要在 rust 中实现本系统对应的 allocator 接口。若要使用动态分配空间，需要完善动态内存分配的系统调用接口，使程序能向系统请求分配内存页。

3. **实验**：实现 `get_tid` 系统调用，使得用户线程可以获取自身的线程 ID。

   **答**：在我使用的 ubuntu 16.04 环境中，经查看 `unistd.h` 中可知 `getpid` 系统调用的 id 为 178，于是选择此作为系统调用号。

   在 `syscall.rs` 下增加新的系统调用函数

   ```rust
   pub(super) fn sys_get_tid() -> SyscallResult {
       let thread: Arc<Thread> = PROCESSOR.get().current_thread();
       SyscallResult::Proceed(thread.id)
   }
   ```

   同时修改 `user` crate 中的部分代码，得到实验结果如下：

   ```
   mod memory initialized
   mod interrupt initialized
   mod driver initialized
   .
   ..
   hello_world
   notebook
   mod fs initialized
   Hello world from user mode program!
   Syscall: The thread id of hello-world is 1.
   Thread 1 exit with code 0
   src/process/processor.rs:101: 'all threads terminated, shutting down'
   ```

   

4. **实验**：将你在实验四（上）实现的 `clone` 改进成为 `sys_clone` 系统调用，使得该系统调用为父进程返回自身的线程 ID，而为子线程返回 0。

   **答**：在我使用的 ubuntu 16.04 环境中，经查看 `unistd.h` 中可知 `sys_clone` 系统调用的 id 为 220，于是选择此作为系统调用号。

   在 `syscall.rs` 下增加新的系统调用函数

   ```rust
   pub(super) fn sys_clone(context: Context) -> SyscallResult {
       let current_thread: Arc<Thread> = PROCESSOR.get().current_thread();
       current_thread.clone_with_context(Some(context));
       SyscallResult::Proceed(current_thread.id)
   }
   ```

   同时在 `thread.rs` 的 `clone_with_context` 函数中增加一行代码：

   ```rust
       /// clone current thread
   pub fn clone_with_context(&self, context: Option<Context>) -> Arc<Thread> {
       ... ...
       // modify the `pc` and `a0`
       let mut context_unwrap: Context = context.expect("fail to load context");
       context_unwrap.set_sp(context_unwrap.sp() - usize::from(self.stack.start) + usize::from(stack.start));
       // return 0 in the sub-thread
       context_unwrap.x[10] = 0;
       ... ...
   }
   ```

   运行测试结果如下：

   在用户程序中，我们克隆自身，并打印线程 id ：

   ```
   mod memory initialized
   mod interrupt initialized
   mod driver initialized
   .
   ..
   hello_world
   notebook
   mod fs initialized
   Hello world from user mode program!
   Clone id is 0
   Syscall: The thread id of hello-world is 2.
   Thread 2 exit with code 0
   Clone id is 1
   Syscall: The thread id of hello-world is 1.
   Thread 1 exit with code 0
   src/process/processor.rs:101: 'all threads terminated, shutting down'
   ```

   可见该线程确实克隆了自身。

5. **实验**：将一个文件打包进用户镜像，并让一个用户进程读取它并打印其内容。需要实现 `sys_open`，将文件描述符加入进程的 `descriptors` 中，然后通过 `sys_read` 来读取。

   **答**：选择 `sys_open` 系统调用的 id 为 1024。实现下面代码：

   ```rust
   pub(super) fn sys_open(filename: &str) -> SyscallResult {
       // 从文件系统中找到程序
       let current_thread: Arc<Thread> = PROCESSOR.get().current_thread();
       let inode = ROOT_INODE.find(filename).unwrap();
       let descriptors: &mut Vec<Arc<dyn INode>> = &mut current_thread.inner().descriptors;
       let ret_id = descriptors.len();
       descriptors.push(inode);
       SyscallResult::Proceed(ret_id as isize)
   }
   ```

   在 `disk.img` 中添加一个 `test` 文件，其中只含有 `123test` 这个字符串，则测试结果：

   ```
   mod memory initialized
   mod interrupt initialized
   mod driver initialized
   .
   ..
   test
   hello_world
   notebook
   mod fs initialized
   Hello world from user mode program!
   test_fd is 2
   [49, 50, 51, 116, 101, 115, 116, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
   Thread 1 exit with code 0
   src/process/processor.rs:101: 'all threads terminated, shutting down'
   ```

   可见我们成功读取了 `test` 文件，并读取了其中字符的 Ascii 码。

6. 挑战实验：实现 `sys_pipe`，返回两个文件描述符，分别为一个管道的读和写端。用户线程调用完 `sys_pipe` 后调用 `sys_fork`，父线程写入管道，子线程可以读取。读取时尽量避免忙等待。

   **答：**忙于其他事务中。