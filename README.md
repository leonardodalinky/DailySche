# DailySche
用于日常打卡 rcore 2020 夏

## 日程总表 表格版

*七月*

|         Sun         |         Mon          |         Tues         |         Wed         |         Thu         |         Fri         |         Sat         |
| :-----------------: | :------------------: | :------------------: | :-----------------: | :-----------------: | :-----------------: | :-----------------: |
| 28<br>([D1](#Day1)) | 29<br/>([D2](#Day2)) | 30<br/>([D3](#Day3)) | 1<br/>([D4](#Day4)) | 2<br/>([D5](#Day5)) | 3<br/>([D6](#Day6)) | 4<br/>([D7](#Day7)) |
| 5<br/>([D8](#Day8)) | 6<br/>([D9](#Day9)) | 7<br/>([D10](#Day10) |          8          |          9          |         10          |         11          |
|         12          |          13          |          14          |         15          |         16          |         17          |         18          |
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

剩下页面置换算法部分未看。打算之后在Tutorial遇到的时候再补上。

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

教程中的` llvm_asm! `宏在现在的rust版本中，已经逐渐逐渐被` asm! `宏取代，于是自己将Lab 0中的相关的内嵌汇编代码修正了。

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