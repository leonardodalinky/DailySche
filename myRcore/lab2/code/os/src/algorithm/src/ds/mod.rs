/// data structure

mod segment_tree;

pub type SegmentTreeResult = Result<isize, &'static str>;

pub trait SegmentTree {
    /// return a new segment tree with boundary
    fn new(left: isize, right: isize) -> Self;
    /// get a available atomic segment
    fn get(&mut self) -> SegmentTreeResult;
    /// return an atomic segment back to segment tree
    fn put(&mut self, i: isize) -> usize;
}

pub use segment_tree::SegmentTreeImpl;