/// implement the segment tree

use super::{SegmentTree, SegmentTreeResult};
use alloc::boxed::Box;
use core::mem::drop;

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

impl SegmentTree for SegmentTreeImpl {
    /// return a new segment tree with boundary
    fn new(left: isize, right: isize) -> Self {
        Self {
            left: left,
            right: right,
            available_size: (right - left) as usize,
            left_child: None,
            right_child: None,
        }
    }
    /// get a available atomic segment
    fn get(&mut self) -> SegmentTreeResult {
        if self.available_size == 0 {
            return Err("not enough segments")
        }
        if self.is_leaf() {
            // leaf node
            self.available_size = 0;
            return Ok(self.left);
        }
        let mid = self.get_mid();
        if let None = self.left_child {
            // left child tree is not used yet
            let p: *mut SegmentTreeImpl = Box::into_raw(Box::new(Self::new(self.left, mid)));
            self.left_child = Some(p);
            self.available_size -= 1;
            return unsafe { p.as_mut().unwrap().get() };
        }
        else {
            // left child tree has been used
            let p: *mut SegmentTreeImpl = self.left_child.clone().unwrap();
            if unsafe { p.as_mut().unwrap().available_size } != 0 {
                self.available_size -= 1;
                return unsafe { p.as_mut().unwrap().get() };
            }
        }

        if let None = self.right_child {
            // right child tree is not used yet
            let p: *mut SegmentTreeImpl = Box::into_raw(Box::new(Self::new(mid, self.right)));
            self.right_child = Some(p);
            self.available_size -= 1;
            return unsafe { p.as_mut().unwrap().get() };
        }
        else {
            // right child tree has been used
            let p: *mut SegmentTreeImpl = self.right_child.clone().unwrap();
            self.available_size -= 1;
            return unsafe { p.as_mut().unwrap().get()};
        }
        
        // unreachable!();
    }
    /// pub back an atomic segment back to segment tree
    /// return the number of returned segment
    fn put(&mut self, i: isize) -> usize {
        if self.is_leaf() {
            if self.available_size == 0 {
                self.available_size = 1;
                return 1;
            }
            return 0;
        }

        let mid = self.get_mid();
        if i < mid {
            if let None = self.left_child {
                return 0;
            }
            else {
                // left child tree has been used
                let p: *mut SegmentTreeImpl = self.left_child.clone().unwrap();
                let ret = unsafe { p.as_mut().unwrap().put(i) };
                self.available_size += ret;
                let r = unsafe { p.as_ref().unwrap() };
                if r.available_size == r.size() {
                    unsafe {
                        drop(Box::from_raw(p));
                    }
                    self.left_child = None;
                }
                return ret;
            }
        }
        else {
            if let None = self.right_child {
                return 0;
            }
            else {
                // right child tree has been used
                let p: *mut SegmentTreeImpl = self.right_child.clone().unwrap();
                let ret = unsafe { p.as_mut().unwrap().put(i) };
                self.available_size += ret;
                let r = unsafe { p.as_ref().unwrap() };
                if r.available_size == r.size() {
                    unsafe {
                        drop(Box::from_raw(p));
                    }
                    self.right_child = None;
                }
                return ret;
            }
        }
    }

    // unreachable!();
}

impl Drop for SegmentTreeImpl {
    fn drop(&mut self) {
        unsafe {
            if !self.left_child.is_none() {
                core::mem::drop(Box::from_raw(self.left_child.unwrap()));
            }
            if !self.right_child.is_none() {
                core::mem::drop(Box::from_raw(self.right_child.unwrap()));
            }
        }
    }
}

impl SegmentTreeImpl {
    /// private
    /// get middle number, which split the segment into [left, mid) and [mid, right)
    fn get_mid(&self) -> isize {
        (self.left + self.right + 1) / 2
    }
    /// private
    /// test if this node is a leaf node
    fn is_leaf(&self) -> bool {
        self.left + 1 == self.right
    }
    /// get the size of segment tree
    pub fn size(&self) -> usize {
        (self.right - self.left) as usize
    }
}
