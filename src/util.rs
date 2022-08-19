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

pub fn equal_escape_u8(a: &[u8], b: &[u8]) -> bool {
    // if !a.contains(&'\\') && !b.contains(&'\\') {
    //     return a == b
    // }

    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] == b'\\' {
            i += 1
        }

        if b[j] == b'\\' {
            j += 1
        }

        if i >= a.len() || j >= b.len() {
            break;
        }

        if a[i] != b[j] {
            return false;
        }

        i += 1;
        j += 1;
    }

    !(j != b.len() || i != a.len())
}
