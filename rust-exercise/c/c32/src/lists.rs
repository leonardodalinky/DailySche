struct node<T> {
    value: T,
    prev: *mut node<T>,
    next: *mut node<T>
}
impl<T> Drop for node<T> {
    fn drop(self: &mut Self) {
        println!("Dropping anode");
    }
}
impl<T> node<T> {
    fn new(value: T, prev: *mut node<T>, next: *mut node<T>) -> Self {
        return Self{value: value, prev: prev, next: next};
    }
}

pub struct list<T> {
    root: *mut node<T>
}
impl<T> Drop for list<T> {
    fn drop(self: &mut Self) {
        let mut this: *mut node<T> = self.root;
        let mut drop: *mut node<T> = std::ptr::null_mut();
        while this != std::ptr::null_mut() {
            drop = this;
            unsafe {this = (*this).next;}
            unsafe {let anode = Box::from_raw(drop);}
        }
    }
}

impl std::fmt::Display for list<f64> {
    fn fmt(self: &Self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[");
        let mut this: *mut node<f64> = self.root;
        while this != std::ptr::null_mut() {
            unsafe {write!(f, "{} ", (*this).value);}
            unsafe {this = (*this).next;}
        }
        write!(f, "]");
        return Ok(());
    }
}

impl<T> list<T> {
    pub fn new() -> Self {
        let p : *mut node<T> = std::ptr::null_mut();
        return Self {root: p};
    }
    pub fn len(self: &Self) -> i32 {
        let mut len: i32 = 0;
        let mut this: *mut node<T> = self.root;
        while this != std::ptr::null_mut()
        {
            len += 1;
            unsafe {
                this = (*this).next;
            }
        }
        return len;
    }
    pub fn getnode(self: &Self, mut i: i32) -> *mut node<T> {
        let mut this: *mut node<T> = self.root;
        let mut next: *mut node<T> = std::ptr::null_mut();
        while i > 0 {
            unsafe {
                next = (*this).next;
            }
            if next == std::ptr::null_mut() {
                return std::ptr::null_mut();
            } else {
                i -= 1;
                this = next;
            }
        }
        return this;

    }
    pub fn insert(mut self: &mut Self, mut i: i32, value: T) {
        let mut next: *mut node<T> = self.getnode(i);
        let mut prev: *mut node<T> = std::ptr::null_mut();

        if i != 0 {
            if next == std::ptr::null_mut() {
                let lastindex: i32 = self.len() - 1;
                prev = self.getnode(lastindex);
            } else {
                unsafe {prev = (*next).prev;}
            }
        }
        let mut anode: Box<node<T>> = Box::new(node::new(value, prev, next));
        let mut this = Box::into_raw(anode);

        if i == 0 {
            self.root = this;
        }

        if next != std::ptr::null_mut() {
            unsafe {(*next).prev = this;}
        }
        if prev != std::ptr::null_mut() {
            unsafe {(*prev).next = this;}
        }
    }
    pub fn erase(mut self: &mut Self, mut i: i32) {
        let mut this: *mut node<T> = self.getnode(i);
        if this == std::ptr::null_mut() {return;}
        let mut prev: *mut node<T> = std::ptr::null_mut();
        let mut next: *mut node<T> = std::ptr::null_mut();
        unsafe {
            prev = (*this).prev;
            next = (*this).next;
        }
        if prev != std::ptr::null_mut() {
            unsafe {(*prev).next = next;}
        }
        if next != std::ptr::null_mut() {
            unsafe {(*next).prev = prev;}
        }
        if i == 0 {
            self.root = next;
        }

        unsafe {let anode: Box<node<T>> = Box::from_raw(this);}
    }
    pub fn getvalue(&self, mut i: i32) -> &'static T {
        let this: *mut node<T> = self.getnode(i);
        unsafe {
            return &(*this).value;
        }
    }
}