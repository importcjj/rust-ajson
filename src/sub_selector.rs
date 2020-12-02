use reader;
use reader::ByteReader;

use std::fmt;
use std::str;
use util;

pub struct SubSelector<'a> {
    pub name: &'a [u8],
    pub path: &'a [u8],
}

impl<'a> SubSelector<'a> {
    pub fn new(name: &'a [u8], path: &'a [u8]) -> SubSelector<'a> {
        SubSelector { name, path }
    }
}

impl<'a> fmt::Debug for SubSelector<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sel")?;
        write!(f, " name=`{}`", str::from_utf8(self.name).unwrap())?;
        write!(f, " path=`{}`", str::from_utf8(self.path).unwrap())?;
        write!(f, ">")
    }
}

fn last_of_name(v: &[u8]) -> &[u8] {
    // for mut i in (0..v.len()).rev() {
    //     match v[i] {
    //         b'\\' => i -= 1,
    //         b'.' => return &v[i + 1..],
    //         _ => (),
    //     }
    // }

    if v.is_empty() {
        return v;
    }

    let mut i = v.len() - 1;
    loop {
        match v[i] {
            b'\\' => i -= 1,
            b'.' => return &v[i + 1..],
            _ => (),
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    v
}

pub fn parse_selectors_from_utf8<'a>(v: &'a [u8]) -> (Vec<SubSelector<'a>>, usize, bool) {
    let mut reader = reader::RefReader::new(v);
    let mut depth = 0;
    let mut start = 0;
    let mut colon = 0;
    let mut sels = Vec::new();

    macro_rules! push_sel {
        () => {{
            if start < reader.position() {
                let sel = if colon == 0 {
                    let key = last_of_name(&v[start..reader.position()]);
                    SubSelector::new(key, &v[start..reader.position()])
                } else {
                    let key = util::trim_u8(&v[start..colon], b'"');
                    SubSelector::new(key, &v[colon + 1..reader.position()])
                };
                sels.push(sel);
            }
        }};
    };

    while let Some(b) = reader.peek() {
        match b {
            b'\\' => {
                reader.next();
            }
            b'"' => {
                reader.read_str_value();
                continue;
            }
            b':' => {
                if depth == 1 {
                    colon = reader.position();
                }
            }
            b',' => {
                if depth == 1 {
                    push_sel!();
                    colon = 0;
                    start = reader.offset();
                }
            }
            b'[' | b'(' | b'{' => {
                depth += 1;
                if depth == 1 {
                    start = reader.position() + 1;
                }
            }

            b']' | b')' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    push_sel!();
                    let length = reader.offset();
                    return (sels, length, true);
                }
            }
            _ => (),
        }

        reader.next();
    }

    (vec![], 0, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_selectors_from_utf8() {
        let path = r#"{name.first,age,murphys:friends.#(last="Murphy")#.first}"#;
        let (sels, length, ok) = parse_selectors_from_utf8(path.as_bytes());
        println!("length {}", length);
        println!("ok {}", ok);
        println!("sels {:?}", sels);

        let path = r#"[name,a]"#;
        let (sels, length, ok) = parse_selectors_from_utf8(path.as_bytes());
        println!("length {}", length);
        println!("ok {}", ok);
        println!("sels {:?}", sels);
    }
}
