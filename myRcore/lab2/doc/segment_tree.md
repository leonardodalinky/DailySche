# 线段树

本文档解释个人对基于线段树的物理页面分配的实现。

## 数据结构

文件位于 `os/src/algorithm/ds/segment_tree.rs` 

```rust
/// a implement for segment tree
/// 
/// range from [left, right)
pub struct SegmentTreeImpl {
    left: isize,
    right: isize,
    available_size: usize,
    left_child: Option<*mut SegmentTreeImpl>,
    right_child: Option<*mut SegmentTreeImpl>,
}
```

在这里，我们线段树的数据结构：

1. `left` ：线段树的区间左端。左端为闭区间。
2. `right` ：线段树的区间右端。右端为开区间。
3. `available_size` ：这棵线段树中剩余可分配的页面数量。
4. `left_child` ：指向左子树，即左半区间，表示为 $[\mathrm{left},\mathrm{mid})$ 。
5. `right_child` ：指向右子树，即右半区间，表示为 $[\mathrm{mid},\mathrm{right})$ 。

## 线段树接口

在线段树中，我们定义其必备方法的接口，文件位于 ``os/src/algorithm/ds/mod.rs``：

```rust
pub trait SegmentTree {
    /// return a new segment tree with boundary
    fn new(left: isize, right: isize) -> Self;
    /// get a available atomic segment
    fn get(&mut self) -> SegmentTreeResult;
    /// return an atomic segment back to segment tree
    fn put(&mut self, i: isize) -> usize;
}
```

在这里，我们为每一个线段树配备了 *创建*、*获取一个可用段*  和 *归还一个借用段* 共3个方法。

## 线段树算法概述

### new() 方法

`new` 方法创建一个新的线段树，其范围由参数指定，且不带子树（即为叶子节点）。

```rust
fn new(left: isize, right: isize) -> Self {
	Self {
		left: left,
		right: right,
		available_size: (right - left) as usize,
		left_child: None,
		right_child: None,
	}
}
```

### get() 方法

`get` 方法返回一个可用的段。算法为递归实现：

1. 检查是否叶子节点。如果是叶子节点且容量为 1，则将该叶子节点的段返回，并更新自己的可用容量。否则进入第 2 步。
2. 若左子树不存在，则创建相应的左子树。若左子树的可用容量大于0，则递归地在左子树上调用 `get` 方法，并更新自己的可用容量。否则，进入第 3 步。
3. 若右子树不存在，则创建相应的右子树。若右子树的可用容量大于0，则递归地在右子树上调用 `get` 方法，并更新自己的可用容量。否则，返回错误。

这个方法可以观察得到，`get` 方法时间复杂度为 $\mathrm{O}(\log n)$。

### put() 方法

`put` 方法归还一个借用的段，并返回过程中成功归还的段的数量。算法为递归实现：

1. 检查是否叶子节点。如果是叶子节点且容量为 0，则将可用容量重置为 1，返回自己所归还的段的数量。如果不是叶子节点，进入第 2 步。
2. 判断待归还的段的所在区间对应的子树。若对应的子树不存在，则返回 0。若存在，则在对应的子树上递归调用 `put` 方法。在子树的 `put` 方法返回后，若子树的可用容量与最大容量相同，则释放该子树（因为此时子树对应的区间段全空闲）。最终，返回过程中成功归还的段的数量。

由于 `put` 方法可以看作 `get` 方法的逆过程，其中 `get` 方法创建出来的子树数量大于等于 `put` 方法释放的子树数量。因此在平摊时间的情况下，`put` 方法的时间复杂度不大于 `get` 方法。

因此，`put` 方法摊还时间复杂度为 $\mathrm{O}(\log n)$。

## 复杂度分析

### 时间复杂度

由上面的分析可知，`get` 和 `put` 方法的时间复杂度均为 $\mathrm{O}(\log n)$。

### 空间复杂度

显然，线段树为二叉树。当树全满的时候，最大空间约为 $2n$。因此空间复杂度理论上为 $\mathrm{O}(n)$。

但是在实际上，由于全满的情况极少出现，且由于我们的线段树的 `put` 方法中有子树释放的过程，实际上的空间使用量应该远远小于理论上的空间复杂度。