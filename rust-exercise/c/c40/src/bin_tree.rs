use std::ptr::NonNull;
use std::cmp::Ordering;

pub struct BinTree<T>
    where T: Default
{
    root: Option<NonNull<BinNode<T>>>,
    size: isize
}

struct BinNode<T>
    where T: Default
{
    pub value: T,
    pub left: Option<NonNull<BinNode<T>>>,
    pub right: Option<NonNull<BinNode<T>>>,
    pub parent: Option<NonNull<BinNode<T>>>
}

impl<T> Drop for BinTree<T>
    where T: Default
{
    fn drop(&mut self) {
        BinNode::delete_r(self.root);
    }
}

impl<T> BinTree<T>
    where T: Default
{
    pub fn new() -> BinTree<T> {
        BinTree {
            root: None,
            size: 0
        }
    }
    pub fn insert(&mut self, t: T)
        where T: Ord
    {
        match self.root {
            None => {
                let mut r = BinNode::new();
                unsafe {
                    r.as_mut().value = t;
                }
                self.root = Some(r);
                return;
            },
            Some(r) => {
                let mut cur = r;
                let mut newNode;
                loop {
                    match unsafe {t.cmp(&((*cur.as_ptr()).value))} {
                        Ordering::Equal => return,
                        Ordering::Less => {
                            if let None = unsafe {(*cur.as_ptr()).left} {
                                newNode = BinNode::new();
                                unsafe {
                                    cur.as_mut().left = Some(newNode);
                                }
                            }
                            else {
                                cur = unsafe {(*cur.as_ptr()).left.unwrap()};
                            }
                        },
                        Ordering::Greater => {
                            if let None = unsafe {(*cur.as_ptr()).right} {
                                newNode = BinNode::new();
                                unsafe {
                                    cur.as_mut().right = Some(newNode);
                                }
                            }
                            else {
                                cur = unsafe {(*cur.as_ptr()).left.unwrap()};
                            }
                        }
                    }
                }
                unsafe {
                    newNode.as_mut().value = t;
                }
            }
        }
    }
    pub fn search(&mut self, t: T) -> bool
        where T: Ord
    {
        match self.root {
            None => {
                return false;
            },
            Some(r) => {
                let mut cur = r;
                loop {
                    match unsafe {t.cmp(&((*cur.as_ptr()).value))} {
                        Ordering::Equal => return true,
                        Ordering::Less => {
                            if let None = unsafe {(*cur.as_ptr()).left} {
                                return false;
                            }
                            else {
                                cur = unsafe {(*cur.as_ptr()).left.unwrap()};
                            }
                        },
                        Ordering::Greater => {
                            if let None = unsafe {(*cur.as_ptr()).right} {
                                return false;
                            }
                            else {
                                cur = unsafe {(*cur.as_ptr()).right.unwrap()};
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T> BinNode<T>
    where T: Default
{
    fn delete_r(pnode: Option<NonNull<BinNode<T>>>) {
        let pnode = match pnode {
            None => return,
            Some(p) => p
        };
        let lc = unsafe {pnode.as_ref().left};
        let rc = unsafe {pnode.as_ref().right};
        unsafe {
            let b = Box::from_raw(pnode.as_ptr());
        }
        Self::delete_r(lc);
        Self::delete_r(rc);
    }
    fn new() -> NonNull<BinNode<T>>
        where T: Default
    {
        let b = Box::new(BinNode{
            value: T::default(),
            left: None,
            right: None,
            parent: None
        });
        return NonNull::new(Box::into_raw(b)).expect("New node fail.");
    }
}