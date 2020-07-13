# DailySche
用于日常打卡 rcore 2020 夏

## 日程总表 表格版

*七月*

|         Sun         |         Mon          |         Tues         |         Wed         |         Thu         |         Fri         |         Sat         |
| :-----------------: | :------------------: | :------------------: | :-----------------: | :-----------------: | :-----------------: | :-----------------: |
| 28<br>([D1](#Day1)) | 29<br/>([D2](#Day2)) | 30<br/>([D3](#Day3)) | 1<br/>([D4](#Day4)) | 2<br/>([D5](#Day5)) | 3<br/>([D6](#Day6)) | 4<br/>([D7](#Day7)) |
| 5<br/>([D8](#Day8)) | 6<br/>([D9](#Day9)) | 7<br/>([D10](#Day10)) |          8<br/>([D11](#Day11))          |          9<br/>([D12](#Day12))          | 10<br/>([D13](#Day13)) | 11<br/>([D14](#Day14)) |
| 12<br/>([D15](#Day15)) |          13<br/>([D16](#Day16))          |          14<br/>([D17](#Day17))          |         15          |         16          |         17          |         18          |
|         19          |          20          |          21          |         22          |         23          |         24          |         25          |
|         26          |          27          |          28          |         29          |         30          |         31          |                     |

## 日程总表 文字版

* [Day 1(2020/06/28)](#Day1)

* [Day 2(2020/06/29)](#Day2)

* [Day 3(2020/06/30)](#Day3)

* [Day 4(2020/07/01)](#Day4)

* [Day 5(2020/07/02)](#Day5)

* [Day 6(2020/07/03)](#Day6)

* [Day 7(2020/07/04)](#Day7)

* [Day 8(2020/07/05)](#Day8)

* [Day 9(2020/07/06)](#Day9)

* [Day 10(2020/07/07)](#Day10)

* [Day 11(2020/07/08)](#Day11)

* [Day 12(2020/07/09)](#Day12)

* [Day 13(2020/07/10)](#Day13)

* [Day 14(2020/07/11)](#Day14)

* [Day 15(2020/07/12)](#Day15)

* [Day 16(2020/07/13)](#Day16)

* [Day 17(2020/07/14)](#Day17)

  <br/>

<span id="Day1"></span>

##  Day 1

### 任务1：完成rustling练习(100%)

前几天和今天大体阅读完Rust官方Tutorial手册，并且完成Rustlings的初学者练习。代码位于项目`/rust-exercise`下。一连学了10个小时，有一点吃不消了。

### 明日预定任务：完成Learn C The Hard Way中15道习题

<br/>

<span id="Day2"></span>

## Day 2

### 任务1：完成Learn C The Hard Way中15道习题（8/15）

用rust模拟c的行为，容易遭到各种操作上的困难，不过再参考了std库中的linked_list的实现后，同时自己实际实现了链表、循环缓冲区和二叉搜索树后，感到自己对rust的unsafe块的理解更进一步。不过这个过程耗费了我许多时间来理解，预定任务也只能完成一半。

### 明日预定任务：继续完成Learn C The Hard Way中剩下的习题

<br/>

<span id="Day3"></span>

## Day 3

### 任务1：完成LCTHW的习题（15/15）

由于LCTHW中有些东西实现起来过于麻烦且无意义，故其中3、4道题换成了自定义的习题。

### 任务2：阅读COAD Risc-V前两章（70%）

因为大部分和已有知识重合，故阅读速度较快。

### 明日预定任务：完成COAD前两章阅读，并阅读RISC-V指令集。

<br/>

<span id="Day4"></span>

## Day 4

### 任务1：阅读完COAD Risc-V前两章（100%）

完成COAD的阅读，对已有知识巩固。

### 任务2：阅读Risc-V中文手册（50%）

阅读完普通部分，准备阅读Privilege部分。

### 明日预定任务：完成Risc-V中文手册

<br/>

<span id="Day5"></span>

## Day 5

### 任务1：Risc-V中文手册

大致过了一遍Risc-V中文手册，发现其中的Privilege部分属于重点，这部分在之后用到的时候更需要深入看一看。

### 任务2：看B站上Thu的Risc-V相关（40%）

有在Bilibili网站上发现前几个月贵系关于Risc-V操作系统和rCore的课程，于是打算先搁置手头上的任务，并看完。之后再打算进行Lab。[视频网址](https://www.bilibili.com/video/BV1GE41157Hc)

### 明日预定任务：学习Thu的Risc-V相关课程

预计视频剩余时长为8小时，需要在两天内完成。

<br/>

<span id="Day6"></span>

## Day 6

### 任务1：Thu的Risc-V相关视频（90%）

剩下内存分配和页面置换算法部分未看。打算之后在Tutorial遇到的时候再补上。

### 任务2：搭设环境

忘记上次重装系统后，Vmware直接没了，就把剩下的时间重新搭了Ubuntu虚拟机。

### 明日预定任务：搭建环境+Lab0

明天搭完环境后，就准备开始Lab了。

<br/>

<span id="Day7"></span>

## Day 7

### 任务1：搭建环境

原生的ubuntu 18.04系统几乎什么库都没有，修修补补装了起来。编译rCore-Tutorial时发现失败，绕了很久的圈子，才知道要先按照Lab 0来写一会，才能慢慢搭建完，感觉到浪费了人生。

### 任务2：Lab 0

完成了Lab 0。

由于在risc-v的asm手册上没有发现` .space `标记，于是使用` .size `实现了相同的初始化栈功能。

```assembly
    .section .bss.stack
    .global boot_stack
boot_stack:
    # 16K 启动栈大小
    .zero 1024 * 16
    .global boot_stack_top
boot_stack_top:
    # 栈结尾
```

教程中的` llvm_asm! `宏在现在的rust版本中，已经逐渐逐渐被新式` asm! `宏取代，于是自己将Lab 0中的相关的内嵌汇编代码修正了。例如，

```rust
/// SBI 调用
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let ret;
    unsafe {
        asm!("ecall",
            inout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which);
    }

    ret
}
```

### 明日预定任务：阅读Privilege文档+Lab1

准备补读之前的Privilege部分的文档，在进入异常机制前需要仔细研读CSR相关部分。

<br/>

<span id="Day8"></span>

## Day 8

### 任务1：阅读Privilege文档

花了一个下午，大致明白了其中的关系。不过硬知识太多，用的时候需要频繁查阅。

### 任务2：Lab1+Lab2

Lab 1跟着教程走，很快就复现了。

问题在于Lab 2。因为我没找到Lab 2对应的源码，估计要git换一下commit号，但我懒惰了。而现有的rCore-Tutorial中的物理内存源码部分已经和虚拟内存部分糅合在一起，所以这部分还需要等明天完成Lab 3后才能正常工作。

### 明日预定任务：Lab2+Lab3

明天搞完Lab2和Lab3后，要整理一下整个内存模块。顺便深入一下线段树和SLAB分配算法。可能还得看一看页置换算法了。这就是个大坑了。

<br/>

<span id="Day9"></span>

## Day 9

### 任务1：Lab 3

这一章完全理清关系，耗了一天。一半原因是IDE的智能类型判断出了问题，而且代码跳转功能不完善，使得我在TUtorial中看了很久才理清了各个文件中的关键所在。

这一章实现了最基础的页表，并且完成了内核重映射。不过也因此里面没有缺页错误的处理，也没有页置换，只能算是实现了最基本的功能，前路漫漫。

没有时间看内存的各种分配算法。决定先建立起对Tutorial内核的整体理解后，再深入到细枝层面钻研。

### 明日预定任务：Lab 4

嗯，准备看线程调度，这也是个大坑。

<br/>

<span id="Day10"></span>

## Day 10

### 任务1：Lab4

Lab 4里面把调度器封装好了，那我就先跳过调度器吧。

这一章节有关于基本线程的运行方式，我基本上理解透彻了。

不过中间有一点卡了我很久，现在想来，问题出现在对RISC V的汇编语法和指令集不熟悉。

我之前认为RISC V中的` JAL rd, offset `跳转语句是需要指定返回地址的寄存器的，而asm源码中出现的是

```assembly
    .globl __interrupt
__interrupt:
	... ...
    jal handle_interrupt	# 不含rd的JAL语句

    .globl __restore
__restore:
	... ...
```

所以我在` Context `的` __interrupt `和` __restore `环节卡了很久，我怎么也想不透为什么进入了` handle_interrupt `函数后，最后还能返回到` __restore `部分。

直到我查了指令集才发现，原来` JAL rd, offset `语句默认指定` x1 `，也就是` ra `（返回地址寄存器）作为返回地址的寄存器，我人直接傻了。也就是相当于源码中` jal handle_interrupt `这一句，默认以下面一句指令的地址，存入` ra `中，相当于调用了一个函数的过程。

虽说上面那个失误卡了我很久，不过得亏这个问题，让我把这部分源码起码翻阅了5次，也加深了我对这一块的印象。

### 任务2：Rust代码练习

最近手都有点生疏了，敲了一道Leetcode中等题练练手。顺便现在的评价标准中，好像对Rust代码实现挺严格的。趁这个期间补几道题吧。

### 明日预定任务：Lab 5

明天准备进入设备树部分了，可以感受一下IO到底是怎么实现的，不过看到有一小节有关于文件系统，感觉很不友好。

<br/>

<span id="Day11"></span>

## Day 11

### 任务1：Lab 5

设备树这一章节，跟着教程走比较容易理解。但是关于设备树和virtio那些概念的规范，完全没看。代码中又都是各种调包和封装，内部细节完全看不到，以后再补一补。

### 明日预定任务：Lab 6

看完这一章，就应该回到Lab 0开始重新写文档了。之前搁置的内存分配等东西准备实现了。

<br/>

<span id="Day12"></span>

## Day 12

### 任务1：Lab 1文档

Lab 6基本都是调库，搁置了。于是写Lab 1的文档吧，不知道写什么，就把自己对各个关键步骤的理解写了下来，并且回答了一下问答题。

除此之外，调整了一下myRcore的文档结构，准备分章节来写。

### 明日预定任务：Lab 2文档

写Lab 2，感觉最近有点累，可能要gap一下。

<br/>

<span id="Day13"></span>

## Day 13

今日我gap一天，搞一些其他东西。和dotnet、docker斗智斗勇一整天，心力憔悴。dotnet所谓的跨平台，难道真的不是跨windows和windows-server吗（笑

### 明日预定任务：Lab 2文档

回到rCore上，认真起来了。

<br/>

<span id="Day14"></span>

## Day 14

### 任务1：Lab 2文档（65%）

完成了Lab 2文档的大部分。打算看一下线段树的物理页分配算法，就这样加进明天的日程里。

### 明日预定任务：内存分配器

看看明天能不能做出一个自己的动态内存分配器出来。

<br/>

<span id="Day15"></span>

## Day 15

### 任务1：Lab 2 内存分配器

遇到了未知的问题，gdb排查两个小时后发现，是内核栈爆栈了，也算理解了为什么 `VectorBitmapAllocator` 中要指定 `4096K` 的大小，应该就是为了防止爆栈。

### 明日预定任务：Lab 1实验题

今天突然发现Lab 1和Lab 2有了实验题，该好好整一整了。

<br/>

<span id="Day16"></span>

## Day 16

### 任务1：Lab 1实验题

完成了。这一章的实验题相对简单。

### 任务2：Lab 2实验题（50%）

做了基础的概念性实验题，实践类型的实验题明天再整。今天也顺便补齐了一下昨天自己写的内存分配器的文档记录，血的教训。

### 明日预定任务：Lab 2实验题

明天学习并实现线段树的物理页面分配算法。

<br/>

<span id="Day17"></span>

## Day 17