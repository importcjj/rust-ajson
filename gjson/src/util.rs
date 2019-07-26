
pub fn trim_space_u8(v: &[u8]) -> &[u8] {
    trim_u8(v, b' ')
}

pub fn trim_u8(v: &[u8], b: u8) -> &[u8] {
    let length = v.len();
    let mut i = 0;
    let mut j = length;
    if length == 0 {
        return v;
    }

    while i < j - 1 {
        if v[i] == b {
            i += 1;
        } else {
            break;
        }
    }

    while j > i {
        if v[j - 1] == b {
            j -= 1;
        } else {
            break;
        }
    }

    &v[i..j]
}

pub fn safe_slice<T>(v: &[T], start: usize, end: usize) -> &[T] {
    // println!("start {}, end {}, len {}",start, end, v.len());
    if start < end && end <= v.len() {
        &v[start..end]
    } else {
        &[]
    }
}