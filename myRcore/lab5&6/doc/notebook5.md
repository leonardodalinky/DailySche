# 设备与文件

限于内容，只可大部分照抄 Tutorial 内容。

## 设备树

### 从哪里读取设备信息

既然我们要实现把数据放在某个存储设备上并让操作系统来读取，首先操作系统就要有一个读取全部已接入设备信息的能力，而设备信息放在哪里又是谁帮我们来做的呢？这个问题其实在 *物理内存探测*  一章中就提到过，在 RISC-V 中，这个一般是由 bootloader，即 OpenSBI 固件完成的。它来完成对于包括物理内存在内的各外设的扫描，将扫描结果以**设备树二进制对象（DTB，Device Tree Blob）**的格式保存在物理内存中的某个地方。而这个放置的物理地址将放在 `a1` 寄存器中，而将会把 HART ID （**HART，Hardware Thread，硬件线程，可以理解为执行的 CPU 核**）放在 `a0` 寄存器上。

在我们之前的函数中并没有使用过这两个参数，如果要使用，我们不需要修改任何入口汇编的代码，只需要给 `rust_main` 函数增加两个参数即可（代码位于 `os/src/main.rs` ）：

```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    memory::init();
    interrupt::init();
    drivers::init(dtb_pa);
    ...
}
```

打印输出一下，`dtb_pa` 变量约在 0x82200000 附近，而内核结束的地址约为 0x80b17000，也就是在我们内核的后面放着，这意味着当我们内核代码超过 32MB 的时候就会出现问题，在更好的实现中，其实 OpenSBI 固件启动的应该是第二级小巧的 Bootloader，而我们现在全部内核内容都在内存中且也没 32MB 那么大，我们暂时不理会这个问题。

### 设备树

上面提到 OpenSBI 固件会把设备信息以设备树的格式放在某个地址上，哪设备树格式究竟是怎样的呢？在各种操作系统中，我们打开设备管理器（Windows）和系统报告（macOS）等内置的系统软件就可以看到我们使用的电脑的设备树，一个典型的设备树如下图所示：

![img](https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-5/pics/device-tree.png)

每个设备在物理上连接到了父设备上最后再通过总线等连接起来构成一整个设备树，在每个节点上都描述了对应设备的信息，如支持的协议是什么类型等等。而操作系统就是通过这些节点上的信息来实现对设备的识别的。

>  **设备节点属性**
>
> 具体而言，一个设备节点上会有几个标准属性，这里简要介绍我们需要用到的几个：
>
> - compatible：该属性指的是该设备的编程模型，一般格式为 "manufacturer,model"，分别指一个出厂标签和具体模型。如 "virtio,mmio" 指的是这个设备通过 virtio 协议、MMIO（内存映射 I/O）方式来驱动
> - model：指的是设备生产商给设备的型号
> - reg：当一些很长的信息或者数据无法用其他标准属性来定义时，可以用 reg 段来自定义存储一些信息
>
> 设备树是一个比较复杂的标准，更多细节可以参考 [Device Tree Reference](https://elinux.org/Device_Tree_Reference)。

### 解析设备树

对于上面的属性，我们不需要自己来实现这件事情，可以直接调用 rCore 中 device_tree 库，然后遍历树上节点即可（代码位于 `os/src/drivers/device_tree.rs` ）：

```rust
/// 递归遍历设备树
fn walk(node: &Node) {
    // 检查设备的协议支持并初始化
    if let Ok(compatible) = node.prop_str("compatible") {
        if compatible == "virtio,mmio" {
            virtio_probe(node);
        }
    }
    // 遍历子树
    for child in node.children.iter() {
        walk(child);
    }
}

/// 整个设备树的 Headers（用于验证和读取）
struct DtbHeader {
    magic: u32,
    size: u32,
}

/// 遍历设备树并初始化设备
pub fn init(dtb_va: VirtualAddress) {
    let header = unsafe { &*(dtb_va.0 as *const DtbHeader) };
    // from_be 是大小端序的转换（from big endian）
    let magic = u32::from_be(header.magic);
    if magic == DEVICE_TREE_MAGIC {
        let size = u32::from_be(header.size);
        // 拷贝数据，加载并遍历
        let data = unsafe { slice::from_raw_parts(dtb_va.0 as *const u8, size as usize) };
        if let Ok(dt) = DeviceTree::load(data) {
            walk(&dt.root);
        }
    }
}
```

注：在开始的时候，有一步来验证 Magic Number，这一步是一个保证系统可靠性的要求，是为了验证这段内存到底是不是设备树。在遍历过程中，一旦发现了一个支持 "`virtio, mmio`" 的设备（其实就是 QEMU 模拟的存储设备），就进入下一步加载驱动的逻辑。

## virtio

### 挂载到 QEMU

为了让 QEMU 挂载上我们虚拟的存储设备，我们这里选了 QEMU 支持的 virtio 协议，需要在 QEMU 运行的时候加入选项，修改 `os/Makefile`：

```Makefile
# 运行 QEMU
qemu: build
    @qemu-system-riscv64 \
            -machine virt \
            -nographic \
            -bios default \
            -device loader,file=$(BIN_FILE),addr=0x80200000 \
            -drive file=$(TEST_IMG),format=raw,id=sfs \      # 模拟存储设备
            -device virtio-blk-device,drive=sfs				# 以 virtio Block Device 的形式挂载到 virtio 总线上
```

其中的 `TEST_IMG` 是特定文件系统格式的磁盘镜像，我们在本小节还不会提及到这个概念，这里可以直接用目录下的测试镜像。

### 什么是 virtio

virtio 起源于 [virtio: Towards a De-Facto Standard For Virtual I/O Devices](https://www.ozlabs.org/~rusty/virtio-spec/virtio-paper.pdf) 这篇论文，主要针对于半虚拟化技术中对通用设备的抽象。

>  **完全虚拟化和半虚拟化**
>
> 在完全虚拟化中，被虚拟的操作系统运行在位于物理机器上的 Hypervisor 之上。被虚拟的操作系统并不知道它已被虚拟化，并且不需要任何更改就可以在该配置下工作。相反，在半虚拟化中，被虚拟的操作系统不仅知道它运行在 Hypervisor 之上，还包含让被虚拟的操作系统更高效地过渡到 Hypervisor 的代码。
>
> 在完全虚拟化模式中，Hypervisor 必须模拟设备硬件，它是在会话的最低级别进行模拟的（例如，网络驱动程序）。尽管在该抽象中模拟很干净，但它同时也是最低效、最复杂的。在半虚拟化模式中，被虚拟的操作系统和 Hypervisor 能够共同合作，让模拟更加高效。半虚拟化方法的缺点是操作系统知道它被虚拟化，并且需要修改才能工作。

具体来说，virtio 的架构如图所示：

![img](https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-5/pics/virtio.gif)

以 virtio 为中心的总线下又挂载了 `virtio-blk`（块设备）总线、`virtio-net`（网络设备）总线、`virtio-pci`（PCI 设备）总线等，本身就构成一个设备树。

### virtio 节点探测

在上一节中，我们实现了对 "virtio,mmio" 的节点的判断，下面我们进一步来区分上面提到的那些 virtio 设备（代码位于 `os/src/drivers/bus/virtio_mmio.rs`）：

```rust
/// 从设备树的某个节点探测 virtio 协议具体类型
pub fn virtio_probe(node: &Node) {
    // reg 属性中包含了描述设备的 Header 的位置
    let reg = match node.prop_raw("reg") {
        Some(reg) => reg,
        _ => return,
    };
    let pa = PhysicalAddress(reg.as_slice().read_be_u64(0).unwrap() as usize);
    let va = VirtualAddress::from(pa);
    let header = unsafe { &mut *(va.0 as *mut VirtIOHeader) };
    // 目前只支持某个特定版本的 virtio 协议
    if !header.verify() {
        return;
    }
    // 判断设备类型
    match header.device_type() {
        DeviceType::Block => virtio_blk::add_driver(header),
        device => println!("unrecognized virtio device: {:?}", device),
    }
}
```

从设备树节点的 reg 信息中可以读出设备更详细信息的放置位置（如：在 0x10000000 - 0x10010000 ），这段区间虽然算是内存区间，但是还记得的吗？我们的物理内存只分布在 0x80000000 到 0x88000000 的空间中，那这段空间哪里来的呢？这就是所谓的**内存映射读写 MMIO（Memory Mapped I/O）**，也就是总线把对设备操作信息传递也映射成了内存的一部分，CPU 操作设备和访问内存的形式没有任何的区别，但读写效果是不同的。大家可以回忆一下计算机组成原理中对串口的访问，这里是一个道理。

所以，为了访问这段地址，我们也需要把它加到页表里面，分别对应在 `os/src/entry.asm` 中的 `boot_page_table` 以及 `os/src/memory/mapping/memory_set.rs` 的新建内核线程中加入了这段地址，使得我们的内核线程可以访问他们。

### virtio_drivers 库

我们在这里不会自己来实现驱动的每一个细节，同样的，我们使用 rCore 中的 virtio_drivers 库，这个库会帮我们通过 MMIO 的方式对设备进行交互，同时我们也需要给这个库提供一些诸如申请物理内存、物理地址虚拟转换等接口。代码位于 `os/src/drivers/bus/virtio_mmio.rs`。

```rust
lazy_static! {
    /// 用于放置给设备 DMA 所用的物理页（[`FrameTracker`]）
    pub static ref TRACKERS: RwLock<BTreeMap<PhysicalAddress, FrameTracker>> =
        RwLock::new(BTreeMap::new());
}

/// 为 DMA 操作申请连续 pages 个物理页（为 [`virtio_drivers`] 库提供）
///
/// 为什么要求连续的物理内存？设备的 DMA 操作只涉及到内存和对应设备
/// 这个过程不会涉及到 CPU 的 MMU 机制，我们只能给设备传递物理地址
/// 而陷于我们之前每次只能分配一个物理页的设计，这里我们假设我们连续分配的地址是连续的
#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysicalAddress {
    let mut pa: PhysicalAddress = Default::default();
    let mut last: PhysicalAddress = Default::default();
    for i in 0..pages {
        let tracker: FrameTracker = FRAME_ALLOCATOR.lock().alloc().unwrap();
        if i == 0 {
            pa = tracker.address();
        } else {
            assert_eq!(last + PAGE_SIZE, tracker.address());
        }
        last = tracker.address();
        TRACKERS.write().insert(last, tracker);
    }
    return pa;
}

/// 为 DMA 操作释放对应的之前申请的连续的物理页（为 [`virtio_drivers`] 库提供）
#[no_mangle]
extern "C" fn virtio_dma_dealloc(pa: PhysicalAddress, pages: usize) -> i32 {
    for i in 0..pages {
        TRACKERS.write().remove(&(pa + i * PAGE_SIZE));
    }
    0
}

/// 将物理地址转为虚拟地址（为 [`virtio_drivers`] 库提供）
///
/// 需要注意，我们在 0xffffffff80200000 到 0xffffffff88000000 是都有对应的物理地址映射的
/// 因为在内核重映射的时候，我们已经把全部的段放进去了
/// 所以物理地址直接加上 Offset 得到的虚拟地址是可以通过任何内核进程的页表来访问的
#[no_mangle]
extern "C" fn virtio_phys_to_virt(pa: PhysicalAddress) -> VirtualAddress {
    VirtualAddress::from(pa)
}

/// 将虚拟地址转为物理地址（为 [`virtio_drivers`] 库提供）
///
/// 需要注意，实现这个函数的目的是告诉 DMA 具体的请求，请求在实现中会放在栈上面
/// 而在我们的实现中，栈是以 Framed 的形式分配的，并不是高地址的线性映射 Linear
/// 为了得到正确的物理地址并告诉 DMA 设备，我们只能查页表
#[no_mangle]
extern "C" fn virtio_virt_to_phys(va: VirtualAddress) -> PhysicalAddress {
    Mapping::lookup(va).unwrap()
}
```

至于为什么要实现这些操作，是因为本身设备是通过**直接内存访问DMA（Direct Memory Access）**技术来实现数据传输的，CPU 只需要给出要传输哪些内容，放在哪段物理内存上面，把请求告诉设备，设备后面的操作就会利用 DMA 而不经过 CPU 直接传输，在传输结束之后，会通过**中断请求 IRQ（Interrupt ReQuest）**技术沿着设备树把"我做完了"这个信息告诉 CPU，CPU 会作为一个中断进一步处理。而为了实现 DMA，我们需要一些请求和内存空间，比如让磁盘把数据传到某个内存段，我们需要告诉设备内存的物理地址（之所以不是虚拟地址是因为 DMA 不会经过 CPU 的 MMU 技术），而且这个物理地址最好是连续的。同时，我们在栈上申请一个请求的结构，这个结构的物理地址也要告诉设备，所以也需要一些虚实地址转换的接口。

现在，我们的 `FRAME_ALLOCATOR` 还只能分配一个帧出来，我们连续调用，暂时先假设他是连续的。同时注意到，为了实现虚实物理转换，我们需要查找页表，很不幸的是 RISC-V 并没有给我提供一个很方便的根据当前页表找到物理地址的指令，所以这里我们在 `os/src/memory/mapping/mapping.rs` 中实现了一个类似的功能。

### 思考

为什么物理地址到虚拟地址转换直接线性映射，而虚拟地址到物理地址却要查表？

**答：**因为从物理地址到虚拟地址的那个转换，是用于在内核操作的时候进行地址转换，而内核自带线性映射。而虚拟地址到物理地址的转换，是用于查询栈中的 IRQ 结构的，而这个栈我们使用 Framed 的形式分配的，不是简单的线性映射，因此需要查表。

## 驱动和块设备驱动

### 什么是块设备

注意到我们在介绍 virtio 时提到了 virtio-blk 设备，这种设备提供了以整块为粒度的读和写操作，一般对应到真实的物理设备是那种硬盘。而之所以是以块为单位是为了加快读写的速度，毕竟硬盘等设备还需要寻道等等操作，一次性读取很大的一块将会节约很多时间。

### 抽象驱动

在写块设备驱动之前，我们先抽象驱动的概念，也方便后面网络设备等的介入。

位于 `os/src/drivers/driver.rs`。

```rust
/// 驱动类型
///
/// 目前只有块设备，可能还有网络、GPU 设备等
#[derive(Debug, Eq, PartialEq)]
pub enum DeviceType {
    Block,
}

/// 驱动的接口
pub trait Driver: Send + Sync {
    /// 设备类型
    fn device_type(&self) -> DeviceType;

    /// 读取某个块到 buf 中（块设备接口）
    fn read_block(&self, _block_id: usize, _buf: &mut [u8]) -> bool {
        unimplemented!("not a block driver")
    }

    /// 将 buf 中的数据写入块中（块设备接口）
    fn write_block(&self, _block_id: usize, _buf: &[u8]) -> bool {
        unimplemented!("not a block driver")
    }
}

lazy_static! {
    /// 所有驱动
    pub static ref DRIVERS: RwLock<Vec<Arc<dyn Driver>>> = RwLock::new(Vec::new());
}
```

这里暂时只有块设备这个种类，不过这样写还是为了方便未来的扩展。

### 抽象块设备

有了驱动的概念，我们进一步抽象块设备：

```rust
/// 块设备抽象（驱动的引用）
pub struct BlockDevice(pub Arc<dyn Driver>);

/// 为 [`BlockDevice`] 实现 [`rcore-fs`] 中 [`BlockDevice`] trait
///
/// 使得文件系统可以通过调用块设备的该接口来读写
impl dev::BlockDevice for BlockDevice {
    /// 每个块的大小（取 2 的对数）
    ///
    /// 这里取 512B 是因为 virtio 驱动对设备的操作粒度为 512B
    const BLOCK_SIZE_LOG2: u8 = 9;

    /// 读取某个块到 buf 中
    fn read_at(&self, block_id: usize, buf: &mut [u8]) -> dev::Result<()> {
        match self.0.read_block(block_id, buf) {
            true => Ok(()),
            false => Err(dev::DevError),
        }
    }

    /// 将 buf 中的数据写入块中
    fn write_at(&self, block_id: usize, buf: &[u8]) -> dev::Result<()> {
        match self.0.write_block(block_id, buf) {
            true => Ok(()),
            false => Err(dev::DevError),
        }
    }

    /// 执行和设备的同步
    ///
    /// 因为我们这里全部为阻塞 I/O 所以不存在同步的问题
    fn sync(&self) -> dev::Result<()> {
        Ok(())
    }
}
```

这里所谓的 `BlockDevice` 其实就是一个 `Driver` 的引用。而且利用 rcore-fs 中提供的 `BlockDevice` trait 实现了为文件系统的接口，实际上是对上传文件系统的连接。

### virtio-blk 块设备驱动

最后，我们来实现 virtio-blk 的驱动（主要通过调用现成的库完成）：

```rust
/// virtio 协议的块设备驱动
struct VirtIOBlkDriver(Mutex<VirtIOBlk<'static>>);

/// 为 [`VirtIOBlkDriver`] 实现 [`Driver`] trait
///
/// 调用了 [`virtio_drivers`] 库，其中规定的块大小为 512B
impl Driver for VirtIOBlkDriver {
    /// 设备类型
    fn device_type(&self) -> DeviceType {
        DeviceType::Block
    }

    /// 读取某个块到 buf 中
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> bool {
        self.0.lock().read_block(block_id, buf).is_ok()
    }

    /// 将 buf 中的数据写入块中
    fn write_block(&self, block_id: usize, buf: &[u8]) -> bool {
        self.0.lock().write_block(block_id, buf).is_ok()
    }
}

/// 将从设备树中读取出的设备信息放到 [`static@DRIVERS`] 中
pub fn add_driver(header: &'static mut VirtIOHeader) {
    let virtio_blk = VirtIOBlk::new(header).expect("failed to init blk driver");
    let driver = Arc::new(VirtIOBlkDriver(Mutex::new(virtio_blk)));
    DRIVERS.write().push(driver.clone());
}
```

需要注意的是，现在的逻辑怎么看都不像是之前提到的**异步 DMA + IRQ 中断**的高级 I/O 操作技术，而更像是阻塞的读取。实际上的确是阻塞的读取，目前 virtio-drivers 库中的代码虽然调用了 DMA，但是返回时还是阻塞的逻辑，我们这里为了简化也没有设计 IRQ 的响应机制。

至此，我们完成了全部的驱动逻辑，我们总结一下目前的设计模式如下所示：

![img](https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-5/pics/design.png)

其中 `Driver` 作为一个核心 trait 为上提供实现，上层也就是 `Driver` 的使用侧（设备的抽象），而下层则是 `Driver` 的实现侧（设备的实现）。而下一个小节，我们将利用这些驱动来实现文件系统。

## 文件系统

之前我们在加载 QEMU 的时候引入了一个磁盘镜像文件，这个文件的打包是由 [rcore-fs-fuse 工具](https://github.com/rcore-os/rcore-fs/tree/master/rcore-fs-fuse) 来完成的，它会根据不同的格式把目录的文件封装成到一个文件系统中，并把文件系统封装为一个磁盘镜像文件。然后我们把这个镜像文件像设备一样挂载在 QEMU 上，QEMU 就把它模拟为一个块设备了。接下来我们需要让操作系统理解块设备里面的文件系统。

### Simple File System

因为文件系统本身比较庞大，我们这里还是用了 rCore 中的文件系统模块 [rcore-fs](https://github.com/rcore-os/rcore-fs)，其中实现了很多格式的文件系统，我们这里选择最简单的 Simple File System（这也是为什么 QEMU 中的设备 id 为 `sfs`），关于文件系统的细节，这里将不展开描述，可以参考[前人的分析](https://rcore-os.github.io/rCore-Tutorial-deploy/docs/lab-5/files/rcore-fs-analysis.pdf)。

不过，为了使用这个模块，一个自然的想法是存取根目录的 `INode`（一个 `INode` 是对一个文件的位置抽象，目录也是文件的一种），后面对于文件系统的操作都可以通过根目录来实现。

### 实现

这里我们用到了我们的老朋友 `lazy_static` 宏，将会在我们第一次使用 `ROOT_INODE` 时进行初始化，而初始化的方式是找到全部设备驱动中的第一个存储设备作为根目录。

位于 `os/src/fs/mod.rs`。

```rust
lazy_static! {
    /// 根文件系统的根目录的 INode
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        // 选择第一个块设备
        for driver in DRIVERS.read().iter() {
            if driver.device_type() == DeviceType::Block {
                let device = BlockDevice(driver.clone());
                // 动态分配一段内存空间作为设备 Cache
                let device_with_cache = Arc::new(BlockCache::new(device, BLOCK_CACHE_CAPACITY));
                return SimpleFileSystem::open(device_with_cache)
                    .expect("failed to open SFS")
                    .root_inode();
            }
        }
        panic!("failed to load fs")
    };
}
```

同时，还可以注意到我们也加入了一个 `BlockCache`，该模块也是 rcore-fs 提供的，提供了一个存储设备在内存 Cache 的抽象，通过调用 `BlockCache::new(device, BLOCK_CACHE_CAPACITY)` 就可以把 `device` 自动变为一个有 Cache 的设备。最后我们用 `SimpleFileSystem::open` 打开并返回根节点即可。

### 测试

终于到了激动人心的测试环节了！我们首先在触发一下 `ROOT_INODE` 的初始化，然后尝试输出一下根目录的内容（位于 `os/src/fs/mod.rs`）：

```rust
/// 打印某个目录的全部文件
pub fn ls(path: &str) {
    let mut id = 0;
    let dir = ROOT_INODE.lookup(path).unwrap();
    print!("files in {}: \n  ", path);
    while let Ok(name) = dir.get_entry(id) {
        id += 1;
        print!("{} ", name);
    }
    print!("\n");
}

/// 触发 [`static@ROOT_INODE`] 的初始化并打印根目录内容
pub fn init() {
    ls("/");
    println!("mod fs initialized");
}
```

最后在主函数中测试初始化，然后测试在另一个内核线程中创建个文件夹，而之所以在另一个线程中做是为了验证我们之前写驱动涉及到的页表的那些操作（位于 `os/src/fs/mod.rs`）：

```rust
/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
#[no_mangle]
pub extern "C" fn rust_main(_hart_id: usize, dtb_pa: PhysicalAddress) -> ! {
    memory::init();
    interrupt::init();
    drivers::init(dtb_pa);
    fs::init();

    let process = Process::new_kernel().unwrap();

    PROCESSOR
        .get()
        .add_thread(Thread::new(process.clone(), simple as usize, Some(&[0])).unwrap());

    // 把多余的 process 引用丢弃掉
    drop(process);

    PROCESSOR.lock().run()
}

/// 测试任何内核线程都可以操作文件系统和驱动
fn simple(id: usize) {
    println!("hello from thread id {}", id);
    // 新建一个目录
    fs::ROOT_INODE
        .create("tmp", rcore_fs::vfs::FileType::Dir, 0o666)
        .expect("failed to mkdir /tmp");
    // 输出根文件目录内容
    fs::ls("/");

    loop {}
}
```

`make run` 一下，你会得到类似的输出：

```
mod memory initialized
mod interrupt initialized
mod driver initialized
files in /:
  . .. temp rust
mod fs initialized
hello from thread id 0
files in /:
  . .. temp rust tmp
100 tick
200 tick
...
```

成功了！我们可以看到系统正确的读出了文件，而且也正确地创建了文件，这为后面用户进程数据的放置提供了很好的保障。