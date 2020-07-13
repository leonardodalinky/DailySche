//! my new dynamic allocator
//! 
#![allow(dead_code)]
use super::RAllocator;
use bit_field::BitArray;
use core::cmp::min;

/// Bitmap 中的位数（8m）
// const BITMAP_SIZE: usize = 0x80_0000;
const BITMAP_SIZE: usize = 8192;

/// 向量分配器的简单实现，每字节用一位表示
pub struct ReverseAllocator {
    /// start pointer
    start_ptr: usize,
    /// end pointer,
    end_ptr: usize,
    /// 容量，单位为 bitmap 中可以使用的位数，即待分配空间的字节数
    capacity: usize,
    /// min existed element's pointer
    min_ex: usize,
    /// 每一位 0 表示空闲
    bitmap: [u8; BITMAP_SIZE / 8],
}

impl RAllocator for ReverseAllocator {
    fn new(start_ptr: usize, capacity: usize) -> Self {
        Self {
            start_ptr: start_ptr,
            end_ptr: start_ptr + min(BITMAP_SIZE, capacity),
            capacity: min(BITMAP_SIZE, capacity),
            min_ex: start_ptr + min(BITMAP_SIZE, capacity),
            bitmap: [0u8; BITMAP_SIZE / 8],
        }
    }
    fn alloc(&mut self, size: usize, align: usize) -> Option<usize> {
        if size == 0 {
            return None;
        }
        // small stuff
        // get a aligned address
        let mask_align = align - 1;
        let mut cur_ptr: usize;
        if size > 128 {
            cur_ptr = (self.min_ex - 1) & (!mask_align);
        }
        else {
            cur_ptr = (self.end_ptr - 1) & (!mask_align);
        }

        let mut record_empty = 0;   // record the size of empty units
        while cur_ptr >= self.start_ptr {
            if !self.bitmap.get_bit(cur_ptr - self.start_ptr) {
                record_empty += 1;
            }
            else {
                record_empty = 0;
            }

            if record_empty >= size && (cur_ptr & mask_align == 0) {
                // could be used as result
                ((cur_ptr - self.start_ptr)..(cur_ptr - self.start_ptr + size)).for_each(|i| self.bitmap.set_bit(i, true));
                self.min_ex = min(cur_ptr, self.min_ex);
                return Some(cur_ptr);
            }

            cur_ptr -= 1;
        }

        return None;
    }
    fn dealloc(&mut self, start_ptr: usize, size: usize, _align: usize) {
        assert!(self.bitmap.get_bit(start_ptr - self.start_ptr));
        let mask_align = _align - 1;
        assert!(start_ptr & mask_align == 0);
        ((start_ptr - self.start_ptr)..(start_ptr - self.start_ptr + size)).for_each(|i| self.bitmap.set_bit(i, false));
        if start_ptr == self.min_ex {
            // renew the min_ex
            let mut cur_ptr = start_ptr;
            while cur_ptr < self.end_ptr {
                if self.bitmap.get_bit(cur_ptr - self.start_ptr) {
                    self.min_ex = cur_ptr;
                    return;
                }

                cur_ptr += 1;
            }
            // all units are vacant
            self.min_ex = self.end_ptr;
        }
    }
}