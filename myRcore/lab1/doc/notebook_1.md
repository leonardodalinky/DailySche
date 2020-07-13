# Lab 1学习报告

## 基础概念

### 中断的概念

中断在我的理解中，是存在于CPU设计中的一种特殊的响应机制。在现代的计算机运行过程中，几乎处处都需要用到中断，比如线程调度、I/O和硬件交互。

这个机制允许我们在计算机在某一时刻中断自己的任务，移交运行的所有权到系统中，并跳转到由计算机系统预先设置好的中断处理程序中，使得并发机制得以成立。

### 中断种类的区分

在实现中断之前，要先确定中断的种类。借用文档中的描述形式，中断的种类分为三种：

* **异常(Exception)**。执行指令时产生的，通常无法预料的错误。例如：访问无效内存地址、执行非法指令（除以零）等。

  有的异常可以恢复，例如缺页异常；有的异常会导致用户程序被终止，例如非法访问。

* **陷阱(Trap)**。陷阱是一系列强行导致中断的指令，例如：**系统调用（syscall）**等。

* **硬件中断(Interrupt)**。前两种都是指令导致的异常，而硬件中断是由 CPU 之外的硬件产生的异步中断，例如：时钟中断、外设发来数据等。

### CSR寄存器

CSR（Control and Status Register）寄存器，负责保存并控制信息。RiscV中预留了至多12位的CSR地址，可以索引4096个寄存器。在中断过程中，我们主要用到下面几个：

**中断时，由硬件填写的寄存器**

* ` sepc `：一个 *SXLEN*（ Supervisor 模式寄存器大小，RiscV64中为64位）位寄存器，存储着触发中断或者异常的指令的地址。` spec `中第0位始终为0。值得注意的是，当中断返回时，会重新执行 ` sepc ` 储存地址中的指令。
* ` scause `：一个 *SXLEN* 位寄存器，储存着触发中断或者异常的原因。其最高位分别指示着中断的类型，0表示异常，1表示中断。剩余的低63位表示触发原因。详细内容可见 RiscV-Privilege-20190608 手册62页。
* ` stval `：一个 *SXLEN* 位寄存器，储存着*额外的*触发中断或者异常的原因。常见于当 ` scause ` 不足以存放所需的信息时， `stval` 则会被使用。

**指导硬件中断的寄存器**

* ` stvec `：一个 *SXLEN* 位寄存器，设置内核态中断处理流程的入口地址。存储了一个基址 BASE 和模式 MODE，其中最低2位储存 MODE，最高62位储存 BASE。
  * MODE 为 0 表示 Direct 模式，即遇到中断便跳转至 BASE 进行执行。
  * MODE 为 1 表示 Vectored 模式，此时 BASE 应当指向一个向量，存有不同处理流程的地址，遇到中断会跳转至 `BASE + 4 * cause` 进行处理流程。

* ` sie `：一个 *MXLEN*（ Machine 模式寄存器大小，RiscV64中为64位）位寄存器，用来控制具体类型中断的使能，例如其中的 STIE 控制时钟中断使能。
* ` sip `：即 Supervisor Interrupt-Pending，和 `sie` 相对应，记录每种中断是否被触发。仅当 `sie` 和 `sip` 的对应位都为 1 时，意味着开中断且已发生中断，这时中断最终触发。
* ` sstatus `：一个 *SXLEN* 位寄存器，具有许多状态位，控制全局中断使能等。详细内容可见 RiscV-Privilege-20190608 手册55页。

### 中断指令

摘录自 rCore-Tutorial 文档。

#### 进入和退出中断

- `ecall`
  触发中断，进入更高一层的中断处理流程之中。用户态进行系统调用进入内核态中断处理流程，内核态进行 SBI 调用进入机器态中断处理流程，使用的都是这条指令。
- `sret`
  从内核态返回用户态，同时将 `pc` 的值设置为 `sepc`。（如果需要返回到 `sepc` 后一条指令，就需要在 `sret` 之前修改 `sepc` 的值）
- `ebreak`
  触发一个断点。
- `mret`
  从机器态返回内核态，同时将 `pc` 的值设置为 `mepc`。

#### 操作 CSR

只有一系列特殊的指令（CSR Instruction）可以读写 CSR。尽管所有模式都可以使用这些指令，用户态只能只读的访问某几个寄存器。

为了让操作 CSR 的指令不被干扰，许多 CSR 指令都是结合了读写的原子操作。不过在实验中，我们只用到几个简单的指令。

- `csrrw dst, csr, src`（CSR Read Write）
  同时读写的原子操作，将指定 CSR 的值写入 `dst`，同时将 `src` 的值写入 CSR。
- `csrr dst, csr`（CSR Read）
  仅读取一个 CSR 寄存器。
- `csrw csr, src`（CSR Write）
  仅写入一个 CSR 寄存器。
- `csrc(i) csr, rs1`（CSR Clear）
  将 CSR 寄存器中指定的位清零，`csrc` 使用通用寄存器作为 mask，`csrci` 则使用立即数。
- `csrs(i) csr, rs1`（CSR Set）
  将 CSR 寄存器中指定的位置 1，`csrc` 使用通用寄存器作为 mask，`csrci` 则使用立即数。

## 程序上下文

为了表示程序运行的状态，我们将保存了程序运行时各个寄存器状态的结构体，成为上下文（Context）。

```rust
use riscv::register::{sstatus::Sstatus, scause::Scause};

#[repr(C)]
pub struct Context {
    pub x: [usize; 32],     // 32 个通用寄存器
    pub sstatus: Sstatus,
    pub sepc: usize
}
```

其中 `#[repr(c)]` 表示结构体按c语言的格式对齐和放置。

## 中断的汇编

### 进入中断

为了实现中断，我们需要操作汇编代码。

首先，先定义一个宏，来方便我们的操作。

```rust
# 宏：将寄存器存到栈上
.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

# 宏：将寄存器从栈中取出
.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm
```

这两个宏，分别简化我们将寄存器存储到栈上的指令。

紧跟着，就是进入中断的过程。

```rust
    .section .text
    .globl __interrupt
# 进入中断
# 保存 Context 并且进入 rust 中的中断处理函数 interrupt::handler::handle_interrupt()
__interrupt:
    # 1. 在栈上开辟 Context 所需的空间
    addi    sp, sp, -34*8
    # 2. 保存通用寄存器，除了 x0（固定为 0）
    SAVE    x1, 1
    addi    x1, sp, 34*8
    # 3. 将原来的 sp（sp 又名 x2）写入 2 位置
    SAVE    x1, 2
    SAVE    x3, 3
    SAVE    x4, 4
    SAVE    x5, 5
    SAVE    x6, 6
    SAVE    x7, 7
    SAVE    x8, 8
    SAVE    x9, 9
    SAVE    x10, 10
    SAVE    x11, 11
    SAVE    x12, 12
    SAVE    x13, 13
    SAVE    x14, 14
    SAVE    x15, 15
    SAVE    x16, 16
    SAVE    x17, 17
    SAVE    x18, 18
    SAVE    x19, 19
    SAVE    x20, 20
    SAVE    x21, 21
    SAVE    x22, 22
    SAVE    x23, 23
    SAVE    x24, 24
    SAVE    x25, 25
    SAVE    x26, 26
    SAVE    x27, 27
    SAVE    x28, 28
    SAVE    x29, 29
    SAVE    x30, 30
    SAVE    x31, 31

    # 4. 取出 CSR 并保存
    csrr    s1, sstatus
    csrr    s2, sepc
    SAVE    s1, 32
    SAVE    s2, 33

    # 5. Context, scause 和 stval 作为参数传入
    mv a0, sp
    csrr a1, scause
    csrr a2, stval
    jal  handle_interrupt
```

在注释 1 的地方，我们首先开辟一个 `34 x 8` 的栈空间（注意栈空间向低地址方向增长）。

从注释 2 开始，我们将所有的通用寄存器保存。在注释 3 的时候，由于 `x2` 寄存器表示栈指针，因此原来的 `x2` 寄存器（开辟栈空间之前）的值，应该由现在的 `sp` 寄存器，加上栈空间的大小得到。

注意注释 4 的阶段，我们按照 `Context` 结构体的顺序，保存 `sstatus` 和 `sepc` 。

从注释 5 开始，我们将 `handle_interrupt` 所需要的参数放在对应位置后，跳转至该函数。

之后就是 `handle_interrupt` 函数体中，对中断机制的实现。值得注意的是，由于是使用 `jal` 命令跳转，进入 `handle_interrupt` 后的 return 地址就是 `jal` 命令的下一句，也就是之后的离开中断过程。

### 离开中断

在这里，我打算先描述离开中断的过程。

```rust
    ... ...
	jal  handle_interrupt

    .globl __restore
# 离开中断
# 从 Context 中恢复所有寄存器，并跳转至 Context 中 sepc 的位置
__restore:
    # 1. 恢复 CSR
    LOAD    s1, 32
    LOAD    s2, 33
    # 2. 思考：为什么不恢复 scause 和 stval？如果不恢复，为什么之前要保存
    csrw    sstatus, s1
    csrw    sepc, s2

    # 3. 恢复通用寄存器
    LOAD    x1, 1
    LOAD    x3, 3
    LOAD    x4, 4
    LOAD    x5, 5
    LOAD    x6, 6
    LOAD    x7, 7
    LOAD    x8, 8
    LOAD    x9, 9
    LOAD    x10, 10
    LOAD    x11, 11
    LOAD    x12, 12
    LOAD    x13, 13
    LOAD    x14, 14
    LOAD    x15, 15
    LOAD    x16, 16
    LOAD    x17, 17
    LOAD    x18, 18
    LOAD    x19, 19
    LOAD    x20, 20
    LOAD    x21, 21
    LOAD    x22, 22
    LOAD    x23, 23
    LOAD    x24, 24
    LOAD    x25, 25
    LOAD    x26, 26
    LOAD    x27, 27
    LOAD    x28, 28
    LOAD    x29, 29
    LOAD    x30, 30
    LOAD    x31, 31

    # 4. 恢复 sp（又名 x2）这里最后恢复是为了上面可以正常使用 LOAD 宏
    LOAD    x2, 2
    sret
```

同样，在注释 1，3，4 处，我们依次还原寄存器。

注释 2 处的**思考题**，之所以不恢复 `stval` 和 ` scause` 是因为，这两个值被保存在了通用寄存器中，并作为参数传递。即便之后出现了中断嵌套，也会被保存在下一次中断所保存的上下文中。同时，这两个值都是由硬件设置的，在中断处理函数运行完成后，其使命就完成了，不需要重复的将其恢复到原状态。

## 中断处理函数

### 开启中断

中断处理函数位于 `os/src/interrupt/handler.rs` 

```rust
use super::context::Context;
use riscv::register::stvec;

global_asm!(include_str!("./interrupt.asm"));

/// 初始化中断处理
///
/// 把中断入口 `__interrupt` 写入 `stvec` 中，并且开启中断使能
pub fn init() {
    unsafe {
        extern "C" {
            /// `interrupt.asm` 中的中断入口
            fn __interrupt();
        }
        // 1. 使用 Direct 模式，将中断入口设置为 `__interrupt`
        stvec::write(__interrupt as usize, stvec::TrapMode::Direct);
    }
}
```

`extern "C"` 块用于声明函数入口，并由链接器来寻找实际地址。

通过注释 1 处，在操作系统初始化时，将汇编中的 `__interrupt` 地址写入 `stvec` 寄存器中，也就完成了中断跳转地址的初始化，并且采用 Direct 跳转模式。

### 触发中断

触发中断的文件位于 `os/src/main.rs`

```rust
...
mod interrupt;
...

/// Rust 的入口函数
///
/// 在 `_start` 为我们进行了一系列准备之后，这是第一个被调用的 Rust 函数
pub extern "C" fn rust_main() -> ! {
    // 初始化各种模块
    interrupt::init();

    unsafe {
        asm!("ebreak");
    };

    unreachable!();
}
```

此处我们使用新式 `asm!` 宏代替 `llvm_asm!` 宏。

这个函数会触发断电中断。

### 开启时钟中断

位于 `os/src/interrupt/timer.rs` 

```rust
//! 预约和处理时钟中断

use crate::sbi::set_timer;
use riscv::register::{time, sie, sstatus};

/// 初始化时钟中断
/// 
/// 开启时钟中断使能，并且预约第一次时钟中断
pub fn init() {
    unsafe {
        // 1. 开启 STIE，允许时钟中断
        sie::set_stimer(); 
        // 2. 开启 SIE（不是 sie 寄存器），允许内核态被中断打断
        sstatus::set_sie();
    }
    // 设置下一次时钟中断
    set_next_timeout();
}
```

此处注释 1 处开启 `STIE` 位，即允许S态被时钟中断。

(摘自 Tutorial)*这里可能引起误解的是 `sstatus::set_sie()`，它的作用是开启 `sstatus` 寄存器中的 SIE 位，与 `sie` 寄存器无关。SIE 位决定中断是否能够打断 supervisor 线程。在这里我们需要允许时钟中断打断 内核态线程，因此置 SIE 位为 1。另外，无论 SIE 位为什么值，中断都可以打断用户态的线程。*

### 实现时间中断处理函数

```rust
use super::timer;
use super::context::Context;
use riscv::register::{
    stvec,
    scause::{Trap, Exception, Interrupt},
};
...

/// 中断的处理入口
/// 
/// `interrupt.asm` 首先保存寄存器至 Context，其作为参数和 scause 以及 stval 一并传入此函数
/// 具体的中断类型需要根据 scause 来推断，然后分别处理
#[no_mangle]
pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) {
    // 可以通过 Debug 来查看发生了什么中断
    // println!("{:x?}", context.scause.cause());
    match scause.cause() {
        // 断点中断（ebreak）
        Trap::Exception(Exception::Breakpoint) => breakpoint(context),
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => supervisor_timer(context),
        // 其他情况未实现
        _ => unimplemented!("{:?}: {:x?}, stval: 0x{:x}", scause.cause(), context, stval),
    }
}

/// 处理 ebreak 断点
/// 
/// 继续执行，其中 `sepc` 增加 2 字节，以跳过当前这条 `ebreak` 指令
fn breakpoint(context: &mut Context) {
    println!("Breakpoint at 0x{:x}", context.sepc);
    context.sepc += 2;
}

/// 处理时钟中断
/// 
/// 目前只会在 [`timer`] 模块中进行计数
fn supervisor_timer(_: &Context) {
    timer::tick();
}
```

上面的几个简单的函数，就实现了最基本的事件处理。

`breakpoint()` 函数实现了对断电中断的处理，其中 `context.sepc += 2;` 这一句表示，中断执行完后，要回到下一句指令继续进行。

`supervisor_timer()` 函数就进行一次计数，并且设置下一次时钟中断的时间。时钟中断在之后的线程调度中，起着重要的作用。

## 实验题

1. 简述：在 `rust_main` 函数中，执行 `ebreak` 命令后至函数结束前，`sp` 寄存器的值是怎样变化的？

   **答**：

   `ebreak` 执行后，首先更新 `sepc` 和 `scause` ，然后进入到 `__interrupt` 函数内。

   在 `__interrupt` 函数内，首先先开辟一个 $34\times 8$ 的栈空间，即 `sp` 的值减去 $34\times 8$ 大小，存放着此时的上下文。

   之后，进入 `handle_interrupt` 函数部分，从这里开始，遵循正常的函数调用和栈使用规则，一直到返回 `__restore` 函数中。

   在 `__restore` 函数内，函数读取完栈中存放的上下文并还原寄存器后，将栈中的 $34\times 8$ 的空间还原，即 `sp` 的值加上 $34\times 8$ 大小后，返回 `rust_main` 函数体内，并执行 `ebreak` 的下一条指令。

2. 回答：如果去掉 `rust_main` 后的 `panic` 会发生什么，为什么？

   **答**：

   程序在从 `entry.asm` 进入 `rust_main` 的时候，其 `ra` 寄存器中存放着在 `_start` 函数中的 `jal rust_main` 指令的下一条指令的地址。

   当去掉 `rust_main` 后的 `panic` 时，就会返回到其 `ra` 寄存器中存放的地址，而在 `entry.asm` 中可以发现， `jal rust_main` 后面已经是其他的段，而且段中的内容将会由链接器来决定，所以我们无法预测之后发生的事情。

3. 实验

   1. 实验：如果程序访问不存在的地址，会得到 `Exception::LoadFault`。模仿捕获 `ebreak` 和时钟中断的方法，捕获 `LoadFault`（之后 `panic` 即可）。

      **思路**：即在 `interrupt::handler::handle_interrupt` 中，增加一个新的分支即可

      ```rust
      pub fn handle_interrupt(context: &mut Context, scause: Scause, stval: usize) {
          // 返回的 Context 必须位于放在内核栈顶
          match scause.cause() {
              ... ...,
              // read illegal address
              Trap::Exception(Exception::LoadFault) => loadfault(context, stval),
              // 其他情况，终止当前线程
              _ => fault(context, scause, stval),
          };
      }
      ```

   2. 实验：在处理异常的过程中，如果程序想要非法访问的地址是 `0x0`，则打印 `SUCCESS!`。
   
      **思路**：在运行时，如果出现 `Exception::LoadFault` 异常并被捕捉的话，则 `stval` 寄存器会存放着非法访问的地址。因此只需要改变一下 `loadfault` 函数体为如下即可：
   
      ```rust
      /// 处理时钟中断
      fn loadfault(_context: &mut Context, stval: usize) {
          if stval == 0x0 {
              println!("SUCCESS!");
          }
          panic!("An illegal address!");
      }
      ```
   
   3. 实验：添加或修改少量代码，使得运行时触发这个异常，并且打印出 `SUCCESS!`。
   
      **思路**：移除 `rust_main` 中的 `panic!` 语句，当从 `rust_main` 返回到汇编代码后，用汇编指令将 `pc` 的值跳转到 `0x0` 处。修改 `entry.asm` 代码如下：
   
      ```assembly
      ... ...
      # 目前 _start 的功能：将预留的栈空间写入 $sp，然后跳转至 rust_main
      _start:
          la sp, boot_stack_top
          jal rust_main
          li t0, 0	# load an immediate 0
          jr t0		# jump to address 0x0
      ... ...
      ```
   
      

## 总结

这一章节，我们完成了最基本的中断处理，还有许多中断形式会在之后的几个章节中，逐步增加。中断在操作系统中的地位十分重要，因此彻底了解中断的工作机制，有助于之后的学习。