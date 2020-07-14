# Lab 2学习报告

## 动态内存分配接口

为了在内核中能够使用动态内存分配，我们需要实现一个动态内存分配的机制。在 Rust 中，我们只需要创建一个实现了 `alloc::alloc::GlobalAlloc` 接口的对象，并用 `#[global_allocator]` 语义对这个对象进行标记，那么当程序需要在内核堆上申请空间是，就会自动向这个对象索取。

其中，`alloc::alloc::GlobalAlloc` 接口定义的方法有（[官方文档](https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html)）：

```rust
pub unsafe trait GlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8;
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 { ... }
    unsafe fn realloc(
        &self, 
        ptr: *mut u8, 
        layout: Layout, 
        new_size: usize
    ) -> *mut u8 { ... }
}
```

可见，方法 `alloc_zeroed` 和 `realloc` 都已经根据 `alloc` 和 `dealloc` 方法实现了，因此，我们只需要实现 `alloc` 和 `dealloc` 方法就可以实现一个动态内存分配器。 `alloc` 函数负责返回一个分配好的内存的起始地址；`dealloc` 负责根据一个起始地址，回收一个已分配好的内存。

当然，这里还有额外的类型 `Layout` ，在[官方文档](https://doc.rust-lang.org/alloc/alloc/struct.Layout.html)中定义。其中有两个属性 `size` 和 `align`：

* `size` 属性表示这一块内存的最小大小
* `align` 属性表示这一块内存起始地址的对齐要求，且必须为2的幂次。

## 动态内存分配

### 动态内存分配算法

动态内存分配算法有 [Buddy System]() 和 [SLAB分配器]() 算法，还有其他的算法。不在此处赘述。

### 支持动态内存分配（Buddy System）

下面代码位于 `os/src/memory/heap.rs`

```rust
// 注释 1
/// 进行动态内存分配所用的堆空间
/// 
/// 大小为 [`KERNEL_HEAP_SIZE`]  
/// 这段空间编译后会被放在操作系统执行程序的 bss 段
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

// 注释 2
/// 堆，动态内存分配器
/// 
/// ### `#[global_allocator]`
/// [`LockedHeap`] 实现了 [`alloc::alloc::GlobalAlloc`] trait，
/// 可以为全局需要用到堆的地方分配空间。例如 `Box` `Arc` 等
#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

/// 初始化操作系统运行时堆空间
pub fn init() {
    // 注释 3
    // 告诉分配器使用这一段预留的空间作为堆
    unsafe {
        HEAP.lock().init(
            HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE
        )
    }
}

/// 空间分配错误的回调，直接 panic 退出
#[alloc_error_handler]
fn alloc_error_handler(_: alloc::alloc::Layout) -> ! {
    panic!("alloc error")
}
```

注释 1 处，开辟了一个 .bss 段中的静态空间，用作内核的堆空间。

注释 2 处，建立一个全局动态内存分配器。

注释 3 处，指定动态内存分配器使用注释 1 处开辟的静态空间作为堆空间。

这里由于文档中使用了前人开发好的分配器，因此不涉及内部细节。

### 支持动态内存分配（自定义）

在自己实现伙伴系统之前，我自己实现了一个自己的简单的分配器，遇到了预想不到的困难。

分配器的规则如下：

* 从内存高地址处往低地址处存放
* 当需要分配的大小超过 128 个字节时，直接从目前分配的地址最小的内存处开始，向内存低地址处搜索可用空间。
* 添加一个位数组，以记录每个字节处是否被分配使用。

这是一个实现起来并不困难的问题，但我在实现的时候忽略了内核栈的大小。开始时，我仍然使用 `8MB` 的大小作为内核堆的大小，因此位数组的大小为 `8MB / 8 = 1MB`。但是在这个分配器初始化的时候，由于分配器内部需要传递一个 `1MB` 大小的位数组，因此超出了内核栈大小，使得运行时产生了难以预料的问题，直到我使用gdb工具调试后，发现 `sp` 的地址去到了明显不正常的地址，才意识到这个问题的严重性。

新的内存分配器可以通过 `main.rs` 中的内存分配检测。

## 物理内存

### 物理内存探测

在文档中，我们对物理内存有这样的描述。

> 操作系统怎样知道物理内存所在的那段物理地址呢？在 RISC-V 中，这个一般是由 Bootloader ，即 OpenSBI 来完成的。它来完成对于包括物理内存在内的各外设的扫描，将扫描结果以 DTB（Device Tree Blob）的格式保存在物理内存中的某个地方。随后 OpenSBI 会将其地址保存在 `a1` 寄存器中，给我们使用。
>
> 这个扫描结果描述了所有外设的信息，当中也包括 QEMU 模拟的 RISC-V Virt 计算机中的物理内存。

QEMU中的内存布局也有详细的描述。

> 通过查看 QEMU 代码中 [`hw/riscv/virt.c`](https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c) 的 `virt_memmap[]` 的定义，可以了解到 QEMU 模拟的 RISC-V Virt 计算机的详细物理内存布局。可以看到，整个物理内存中有不少内存空洞（即含义为 unmapped 的地址空间），也有很多外设特定的地址空间，现在我们看不懂没有关系，后面会慢慢涉及到。目前只需关心最后一块含义为 DRAM 的地址空间，这就是 OS 将要管理的 128 MB 的内存空间。

|  起始地址  |  终止地址  | 含义                        |
| :--------: | :--------: | :-------------------------- |
|    0x0     |   0x100    | QEMU VIRT_DEBUG             |
|   0x100    |   0x1000   | unmapped                    |
|   0x1000   |  0x12000   | QEMU MROM                   |
|  0x12000   |  0x100000  | unmapped                    |
|  0x100000  |  0x101000  | QEMU VIRT_TEST              |
|  0x101000  | 0x2000000  | unmapped                    |
| 0x2000000  | 0x2010000  | QEMU VIRT_CLINT             |
| 0x2010000  | 0x3000000  | unmapped                    |
| 0x3000000  | 0x3010000  | QEMU VIRT_PCIE_PIO          |
| 0x3010000  | 0xc000000  | unmapped                    |
| 0xc000000  | 0x10000000 | QEMU VIRT_PLIC              |
| 0x10000000 | 0x10000100 | QEMU VIRT_UART0             |
| 0x10000100 | 0x10001000 | unmapped                    |
| 0x10001000 | 0x10002000 | QEMU VIRT_VIRTIO            |
| 0x10002000 | 0x20000000 | unmapped                    |
| 0x20000000 | 0x24000000 | QEMU VIRT_FLASH             |
| 0x24000000 | 0x30000000 | unmapped                    |
| 0x30000000 | 0x40000000 | QEMU VIRT_PCIE_ECAM         |
| 0x40000000 | 0x80000000 | QEMU VIRT_PCIE_MMIO         |
| 0x80000000 | 0x88000000 | DRAM 缺省 128MB，大小可配置 |

在 QEMU 中，可以使用 `-m` 指定 RAM 的大小，默认是 128 MB 。因此，默认的 DRAM 物理内存地址范围就是 [0x80000000, 0x88000000)。

### 物理内存结构体

由于在物理内存之后，要涉及虚拟内存，所以我们用 `PhysicalAddress` 的类来存储物理内存地址，其中存储着一个 `usize` 类型量。这部分代码在 `os/src/memory/address.rs` 文件，在这里我们仔细解读一下。

首先，是结构体定义。

```rust
use super::config::PAGE_SIZE;

/// 物理地址
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PhysicalAddress(pub usize);

/// 物理页号
#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PhysicalPageNumber(pub usize);
```

可见物理地址与物理页号，内部都只存着一个 `usize` 类型量，并且为了方便，还默认实现了众多常用接口，注意物理地址与页号都实现了 `Copy` 接口，因此对其赋值会采取复制语义，而不是移动语义。

然后，是各种物理地址、物理页号和 `usize` 的转换，以及他们之间的各种运算，此处省略。

### 硬编码内存结束地址

下面代码位于 `os/src/memory/config.rs`

```rust
lazy_static! {
    /// 内核代码结束的地址，即可以用来分配的内存起始地址
    ///
    /// 因为 Rust 语言限制，我们只能将其作为一个运行时求值的 static 变量，而不能作为 const
    pub static ref KERNEL_END_ADDRESS: PhysicalAddress = PhysicalAddress(kernel_end as usize);
}

extern "C" {
    /// 由 `linker.ld` 指定的内核代码结束位置
    ///
    /// 作为变量存在 [`KERNEL_END_ADDRESS`]
    fn kernel_end();
}
```

在上面，我们通过链接器，将内核代码的结束地址存入static变量中。

## 物理内存管理

### 物理页

>通常，我们在分配物理内存时并不是以字节为单位，而是以一**物理页(Frame)**，即连续的 4 KB 字节为单位分配。我们希望用物理页号（Physical Page Number，PPN）来代表一物理页，实际上代表物理地址范围在 $[\text{PPN}\times 4\text{KB},(\text{PPN}+1)\times 4\text{KB})$ 的一物理页。
>
>不难看出，物理页号与物理页形成一一映射。为了能够使用物理页号这种表达方式，每个物理页的开头地址必须是 4 KB 的倍数。但这也给了我们一个方便：对于一个物理地址，其除以 4096（或者说右移 12 位）的商即为这个物理地址所在的物理页号。
>
>同样的，我们还是用一个新的结构来封装一下物理页，一是为了和其他类型地址作区分；二是我们可以同时实现一些页帧和地址相互转换的功能。为了后面的方便，我们也把虚拟地址和虚拟页（概念还没有涉及，后面的指导会进一步讲解）一并实现出来，这部分代码请参考 `os/src/memory/address.rs`。
>
>同时，我们也需要在 `os/src/memory/config.rs` 中加入相关的设置：

```rust
/// 页 / 帧大小，必须是 2^n
pub const PAGE_SIZE: usize = 4096;

/// 可以访问的内存区域起始地址
pub const MEMORY_START_ADDRESS: PhysicalAddress = PhysicalAddress(0x8000_0000);
/// 可以访问的内存区域结束地址
pub const MEMORY_END_ADDRESS: PhysicalAddress = PhysicalAddress(0x8800_0000);
```

这里的页帧大小为常见的 12 位，也定义了内存的 128M 的上下限。事实上，在本章节结束时，`os/src/memory/address.rs` 中并没有实现虚拟内存相关部分。

### 分配和回收

```rust
/// 分配出的物理页
///
/// # `Tracker` 是什么？
/// 太长不看
/// > 可以理解为 [`Box`](alloc::boxed::Box)，而区别在于，其空间不是分配在堆上，
/// > 而是直接在内存中划一片（一个物理页）。
///
/// 在我们实现操作系统的过程中，会经常遇到「指定一块内存区域作为某种用处」的情况。
/// 此时，我们说这块内存可以用，但是因为它不在堆栈上，Rust 编译器并不知道它是什么，所以
/// 我们需要 unsafe 地将其转换为 `&'static mut T` 的形式（`'static` 一般可以省略）。
///
/// 但是，比如我们用一块内存来作为页表，而当这个页表我们不再需要的时候，就应当释放空间。
/// 我们其实更需要一个像「创建一个有生命期的对象」一样的模式来使用这块内存。因此，
/// 我们不妨用 `Tracker` 类型来封装这样一个 `&'static mut` 引用。
///
/// 使用 `Tracker` 其实就很像使用一个 smart pointer。如果需要引用计数，
/// 就在外面再套一层 [`Arc`](alloc::sync::Arc) 就好
pub struct FrameTracker(PhysicalAddress);

impl FrameTracker {
    /// 帧的物理地址
    pub fn address(&self) -> PhysicalAddress {
        self.0
    }
    /// 帧的物理页号
    pub fn page_number(&self) -> PhysicalPageNumber {
        PhysicalPageNumber::from(self.0)
    }
}

/// 帧在释放时会放回 [`static@FRAME_ALLOCATOR`] 的空闲链表中
impl Drop for FrameTracker {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self);
    }
}
```

上面的注释非常的长，但事实上要点其实非常简单。

我们之所以实现这样一个 `FrameTracker` 类，用它来包装我们分配出来的物理地址，其实是为了让 Rust 通过所有权机制来自动帮我们管理内存的释放。因为当一个 `FrameTracker` 不再被任何东西所有的时候，就会触发 Rust 的 Drop 机制，也就帮我们自动管理了内存。

当然，我们也注意到在 `drop` 函数中，有一个 `FRAME_ALLOCATOR` ，这个是我们下一小节将讲的页帧分配器。

### 页帧分配

> 最后，我们封装一个物理页分配器，为了符合更 Rust 规范的设计，这个分配器将不涉及任何的具体算法，具体的算法将用一个名为 `Allocator` 的 Rust trait 封装起来，而我们的 `FrameAllocator` 会依赖于具体的 trait 实现例化。

代码位于 `os/src/memory/frame/allocator.rs`

```rust
lazy_static! {
    // 注释 1
    /// 帧分配器
    pub static ref FRAME_ALLOCATOR: Mutex<FrameAllocator<AllocatorImpl>> = 
    Mutex::new(
        FrameAllocator::new(
            Range::from(
                PhysicalPageNumber::ceil(
                    PhysicalAddress::from(*KERNEL_END_ADDRESS)
                )
                ..PhysicalPageNumber::floor(MEMORY_END_ADDRESS)
    		)
    	)
    );
}

// 注释 2
/// 基于线段树的帧分配 / 回收
pub struct FrameAllocator<T: Allocator> {
    /// 可用区间的起始
    start_ppn: PhysicalPageNumber,
    /// 分配器
    allocator: T,
}

impl<T: Allocator> FrameAllocator<T> {
    /// 创建对象
    pub fn new(range: impl Into<Range<PhysicalPageNumber>> + Copy) -> Self {
        FrameAllocator {
            start_ppn: range.into().start,
            allocator: T::new(range.into().len()),
        }
    }

    // 注释 3
    /// 分配帧，如果没有剩余则返回 `Err`
    pub fn alloc(&mut self) -> MemoryResult<FrameTracker> {
        self.allocator
            .alloc()// return Option<usize>
        		// transform Option<usize> to Result<usize, &str>
            .ok_or("no available frame to allocate")
				// transform Result<usize, &str> to Result<FrameTracker, &str>
            .map(|offset| FrameTracker::from(self.start_ppn + offset))
    }

    /// 将被释放的帧添加到空闲列表的尾部
    ///
    /// 这个函数会在 [`FrameTracker`] 被 drop 时自动调用，不应在其他地方调用
    pub(super) fn dealloc(&mut self, frame: &FrameTracker) {
        self.allocator.dealloc(frame.page_number() - self.start_ppn);
    }
}
```

注释 1 处，为了方便理清括号嵌套关系，于是改写了一下定义的形式。

注释 2 处，我们用 `FrameAllocator` 包装了另一个实现了 `Allocator` 接口的分配器，之后的内存分配操作，实际上都转交给了另一个实现了 `Allocator` 接口的分配器。

注释 3 处的类型转换比较复杂，也在此处做出了详细转换过程。

这个 `Allocator` 接口的定义如下：

```rust
/// 分配器：固定容量，每次分配 / 回收一个元素
pub trait Allocator {
    /// 给定容量，创建分配器
    fn new(capacity: usize) -> Self;
    /// 分配一个元素，无法分配则返回 `None`
    fn alloc(&mut self) -> Option<usize>;
    /// 回收一个元素
    fn dealloc(&mut self, index: usize);
}
```

引用文档的话，可知 `FrameAllocator` 的各种实现细节。

> 并在 `os/src/data_structure/` 中分别实现了链表和线段树算法，具体内容可以参考代码。
>
> 我们注意到，我们使用了 `lazy_static!` 和 `Mutex` 来包装分配器。需要知道，对于 `static mut` 类型的修改操作是 unsafe 的。我们之后会提到线程的概念，对于静态数据，所有的线程都能访问。当一个线程正在访问这段数据的时候，如果另一个线程也来访问，就可能会产生冲突，并带来难以预测的结果。
>
> 所以我们的方法是使用 `spin::Mutex` 给这段数据加一把锁，一个线程试图通过 `lock()` 打开锁来获取内部数据的可变引用，如果钥匙被别的线程所占用，那么这个线程就会一直卡在这里；直到那个占用了钥匙的线程对内部数据的访问结束，锁被释放，将钥匙交还出来，被卡住的那个线程拿到了钥匙，就可打开锁获取内部引用，访问内部数据。
>
> 这里使用的是 `spin::Mutex`，我们需要在 `os/Cargo.toml` 中添加依赖。幸运的是，它也无需任何操作系统支持（即支持 `no_std`），我们可以放心使用。

好一个 `spin::Mutex` 锁，减轻了我们之后多线程内存分配的负担。

### Allocator的算法

TODO

### 测试运行

>  最后，在把新写的模块加载进来，并在 main 函数中进行简单的测试：

```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();

    // 物理页分配
    for _ in 0..2 {
        let frame_0 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        let frame_1 = match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
        };
        println!("{} and {}", frame_0.address(), frame_1.address());
    }

    loop{}
}
```

> 可以看到类似这样的输出：

```
PhysicalAddress(0x80a14000) and PhysicalAddress(0x80a15000)
PhysicalAddress(0x80a14000) and PhysicalAddress(0x80a15000)
```

可见在内存管理机制下，两次分配的地址是一样的，证明内存被重用并重新分配了。

## 思考

若运行下面的代码：

```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();
    memory::init();

    // 物理页分配
    match memory::frame::FRAME_ALLOCATOR.lock().alloc() {
            Result::Ok(frame_tracker) => frame_tracker,
            Result::Err(err) => panic!("{}", err)
    };

    loop{}
}
```

上述代码可能导致问题。

问题的来源是 `FRAME_ALLOCATOR` 在 match 块中被加锁，而 `frame_tracker` 却在 match 块中被 drop，drop 的时候却需要获得 `FRAME_ALLOCATOR` 的锁，这样就形成了死锁。

## 实验题

1. 回答：我们在动态内存分配中实现了一个堆，它允许我们在内核代码中使用动态分配的内存，例如 `Vec` `Box` 等。那么，如果我们在实现这个堆的过程中使用 `Vec` 而不是 `[u8]`，会出现什么结果？

   **答**：显然这是一个死循环。

2. 实验

   1. 回答：`algorithm/src/allocator` 下有一个 `Allocator` trait，我们之前用它实现了物理页面分配。这个算法的时间和空间复杂度是什么？

      **答**：

      观察 `AllocatorImpl` 代码中的 `alloc` 和 `dealloc` 函数体，可以发现其中的操作都是对于 `Vec` 的 `push` 和 `pop` 操作，这两个操作都是 $\mathrm{O}(1)$ 的时间复杂度。

      至于空间复杂度，由于每一个物理页，都有可能存在一个对应的 $[PPN, PPN+1)$ 区间，因此其空间复杂度为 $\mathrm{O}(n)$

   2. 实现基于线段树的物理页面分配算法

      **思路**：根据线段树，在已有的接口上进行改进。详情请看同目录下的 `segment_tree.md` 文件。

3. 挑战实验（选做）

   目前无打算，若有可能，以后实现。自己在上面曾实现过一个简易的动态内存分配器。