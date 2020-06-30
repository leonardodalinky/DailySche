// Quick sort
use std::cmp::Ordering;

fn qsort<T>(arr: &mut [T])
    where T: Ord
{
    let len = arr.len();
    unsafe {
        _qsort(arr.as_mut_ptr(), 0, len as isize);
    }
}

unsafe fn _qsort<T>(arr: *mut T, l: isize, r: isize)
    where T: Ord
{
    if l + 1 >= r {
        return;
    }

    let mut al_len: isize = l;
    let ptr = arr as *mut T;
    let pivot = ptr.add((r - 1) as usize).read();
    for index in l..r {
        match ptr.add(index as usize).read().cmp(&pivot) {
            Ordering::Less => {
                let tmp = ptr.add(al_len as usize).read();
                ptr.add(al_len as usize).write(ptr.add(index as usize).read());
                ptr.add(index as usize).write(tmp);
                al_len += 1;
            }
            _ => {}
        }
    }
    let tmp = ptr.add(al_len as usize).read();
    ptr.add(al_len as usize).write(pivot);
    ptr.add((r - 1) as usize).write(tmp);

    // recursively
    _qsort(arr, l, al_len);
    _qsort(arr, al_len + 1, r);
}

fn main() {
    let mut a = [2, 1,3,114,91, 19];
    qsort(&mut a);
    println!("{:?}", a);
}
