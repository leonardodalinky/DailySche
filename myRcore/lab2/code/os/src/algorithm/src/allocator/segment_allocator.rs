//! an allocator implemented by segment tree [`SegmentAllocator`]

use super::Allocator;
use crate::ds::{SegmentTree, SegmentTreeImpl};

/// an allocator implemented by segment tree
///
/// 每个元素 tuple `(start, end)` 表示 [start, end) 区间为可用。
pub struct SegmentAllocator {
    segment_tree: SegmentTreeImpl,
}

impl Allocator for SegmentAllocator {
    fn new(capacity: usize) -> Self {
        Self {
            segment_tree: SegmentTreeImpl::new(0, capacity as isize),
        }
    }

    fn alloc(&mut self) -> Option<usize> {
        self.segment_tree.get().ok().map(|u| { u as usize })
    }

    fn dealloc(&mut self, index: usize) {
        self.segment_tree.put(index as isize);
    }
}

unsafe impl Send for SegmentAllocator {}
