#[derive(Debug)]
pub struct SubSelector<'a> {
    pub name: &'a [u8],
    pub path: &'a [u8],
}

impl<'a> SubSelector<'a> {
    pub fn new(name: &'a [u8], path: &'a [u8]) -> SubSelector<'a> {
        SubSelector { name, path }
    }
}

fn last_of_name(chars: &[char]) -> &[char] {
    for mut i in (0..chars.len()).rev() {
        match chars[i] {
            '\\' => i -= 1,
            '.' => return &chars[i+1..],
            _ => (),
        }
    }

    return chars;
}

pub fn parse_selectors_from_utf8<'a>(v: &'a [u8]) {
    
}