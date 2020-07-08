
pub fn diving_board(shorter: i32, longer: i32, k: i32) -> Vec<i32> {
    let mut base = shorter * k;
    let offset = longer - shorter;
    let mut ret = Vec::new();
    if k <= 0 {
        return ret;
    }
    else if offset == 0 {
        ret.push(base);
        return ret;
    }
    for i in 0..k+1 {
        ret.push(base);
        base += offset;
    }

    ret
}

fn main() {
    // Nothing happen.
}