use std::default::Default;
pub struct RingBuffer<T>
{
    length: i32,
    size: i32,
    start: i32,
    end: i32,
    buf: *mut T
}

impl<T> RingBuffer<T>
    where T: Default
{
    pub fn new(size: i32) -> RingBuffer<T> {
        let mut b: Vec<T> = Vec::new();
        b.resize_with(size as usize, T::default);
        let mut p = Box::into_raw(b.into_boxed_slice()) as *mut T;
        RingBuffer {
            length: size,
            size: 0,
            start: 0,
            end: 0,
            buf: p
        }
    }

    pub fn write(&mut self, n: T) -> Result<i32, i32> {
        if self.size >= self.length {
            return Err(1);
        }

        unsafe {self.buf.add(self.end as usize).write(n);}
        self.end = self.index_in_range(self.end + 1);
        self.size += 1;
        return Ok(1);
    }

    pub fn read(&mut self) -> Result<T, i32> {
        if self.size >= self.length {
            return Err(1);
        }

        let ans = unsafe {
            let tmp: T = self.buf.add(self.start as usize).read();
            Ok(tmp)
        };
        self.start = self.index_in_range(self.start + 1);
        self.size -= 1;
        return ans;
    }

    fn index_in_range(&self, index: i32) -> i32 {
        if index >= self.length {
            index % self.length
        }
        else if index < 0 {
            index % self.length + self.length
        }
        else {
            index
        }
    }
}

impl<T> Drop for RingBuffer<T> {
    fn drop(&mut self) {
        unsafe {
            let b = Box::from_raw(self.buf);
        }
    }
}