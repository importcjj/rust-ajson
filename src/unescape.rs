use std::u32;

pub fn unescape(v: &[u8]) -> String {
    let mut s = Vec::with_capacity(v.len());
    let mut i = 0;
    while i < v.len() {
        match v[i] {
            b'\\' => {
                i += 1;
                if i >= v.len() {
                    break;
                }

                match v[i] {
                    b'"' | b'\\' | b'/' => s.push(v[i]),
                    // https://github.com/rust-lang/rfcs/issues/751
                    b'b' => s.push(0x08),
                    b'f' => s.push(0x0C),
                    b'n' => s.push(b'\n'),
                    b'r' => s.push(b'\r'),
                    b't' => s.push(b'\t'),
                    b'u' => {
                        let j = i;
                        if i + 5 > v.len() {
                            for c in &v[j - 1..] {
                                s.push(*c);
                            }
                            break;
                        }

                        let mut codepoint = u8s_to_u32(&v[i + 1..i + 5]);
                        let mut parse_ok = false;
                        i += 5;

                        let mut is_surrogate_pair = true;
                        // UTF-16 surrogate pairs
                        // Surrogates are characters in the Unicode range U+D800—U+DFFF (2,048 code points): it is also the Unicode category “surrogate” (Cs). The range is composed of two parts:
                        // U+D800—U+DBFF (1,024 code points): high surrogates
                        // U+DC00—U+DFFF (1,024 code points): low surrogates
                        match codepoint {
                            0x0000...0xD7FF => (),
                            0xD800...0xDBFF => {
                                if i + 5 < v.len() && v[i] == b'\\' {
                                    codepoint -= 0xD800;
                                    codepoint <<= 10;
                                    let lower = u8s_to_u32(&v[i + 2..i + 6]);
                                    if let 0xDC00...0xDFFF = lower {
                                        codepoint = (codepoint | lower - 0xDC00) + 0x010000;
                                    }
                                    i += 6;
                                } else {
                                    i += 2;
                                }
                            }
                            0xE000...0xFFFF => (),
                            _ => is_surrogate_pair = false,
                        };

                        if is_surrogate_pair {
                            parse_ok = write_codepoint(codepoint, &mut s);
                        }

                        if !parse_ok {
                            for c in &v[j - 1..i] {
                                s.push(*c)
                            }
                            // if i> j+6 {
                            //     s.push(b'\\');
                            //     for c in &v[j+5..i] {
                            //         s.push(*c)
                            //     }
                            // }
                        }
                        i -= 1;
                    }
                    b @ _ => {
                        s.push(b'\\');
                        s.push(b);
                    }
                }
            }
            c @ _ => s.push(c),
        }
        i += 1;
    }

    String::from_utf8_lossy(&s).to_string()
}

fn u8s_to_u32(v: &[u8]) -> u32 {
    u8_to_u32(v[0]) << 12 | u8_to_u32(v[1]) << 8 | u8_to_u32(v[2]) << 4 | u8_to_u32(v[3])
}

fn u8_to_u32(b: u8) -> u32 {
    let r = match b {
        b'0'...b'9' => (b - b'0'),
        b'a'...b'f' => (b + 10 - b'a'),
        b'A'...b'F' => (b + 10 - b'A'),
        _ => panic!("unexpected byte"),
    };
    r as u32
}

fn write_codepoint<'a>(codepoint: u32, buffer: &mut Vec<u8>) -> bool {
    match codepoint {
        0x0000...0x007F => buffer.push(codepoint as u8),
        0x0080...0x07FF => buffer.extend_from_slice(&[
            (((codepoint >> 6) as u8) & 0x1F) | 0xC0,
            ((codepoint as u8) & 0x3F) | 0x80,
        ]),
        0x0800...0xFFFF => buffer.extend_from_slice(&[
            (((codepoint >> 12) as u8) & 0x0F) | 0xE0,
            (((codepoint >> 6) as u8) & 0x3F) | 0x80,
            ((codepoint as u8) & 0x3F) | 0x80,
        ]),
        0x10000...0x10FFFF => buffer.extend_from_slice(&[
            (((codepoint >> 18) as u8) & 0x07) | 0xF0,
            (((codepoint >> 12) as u8) & 0x3F) | 0x80,
            (((codepoint >> 6) as u8) & 0x3F) | 0x80,
            ((codepoint as u8) & 0x3F) | 0x80,
        ]),
        _ => return false,
    };

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unescape() {
        println!(
            "{}",
            unescape(r#"\ud83d\udd13, \ud83c\udfc3 OK: \u2764\ufe0f"#.as_bytes())
        );
    }
}
