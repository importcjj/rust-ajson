use crate::unescape;
use crate::value::Value;
use crate::Number;
use crate::Result;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum Element<'a> {
    String(&'a [u8]),
    Object(&'a [u8]),
    Array(&'a [u8]),
    Null(&'a [u8]),
    Boolean(&'a [u8]),
    Number(&'a [u8]),
    Count(usize),
    List(Vec<Element<'a>>),
    Map(HashMap<&'a str, Element<'a>>),
}

impl<'a> Element<'a> {
    pub fn to_value(&self) -> Value<'a> {
        match &self {
            Element::String(buf) => {
                let s = unescape(&buf[1..buf.len() - 1]);
                Value::String(Cow::Owned(s))
            }
            Element::Object(s) => {
                let s = unsafe { std::str::from_utf8_unchecked(s) };
                Value::Object(Cow::Borrowed(s))
            }
            Element::Array(s) => {
                let s = unsafe { std::str::from_utf8_unchecked(s) };
                Value::Array(Cow::Borrowed(s))
            }
            Element::Boolean(buf) => match buf[0] {
                b't' => Value::Boolean(true),
                _ => Value::Boolean(false),
            },
            Element::Number(buf) => {
                let n = Number::from(*buf);
                Value::Number(n)
            }
            Element::Count(num) => Value::Usize(*num),
            Element::List(elements) => {
                let mut array_string = String::new();
                array_string.push('[');
                let size = elements.len();

                for (i, element) in elements.iter().enumerate() {
                    element.write_to_string_buffer(&mut array_string);
                    if i < size - 1 {
                        array_string.push(',');
                    }
                }

                array_string.push(']');
                Value::Array(Cow::Owned(array_string))
            }
            Element::Map(elements) => {
                let mut object_string = String::new();
                object_string.push('{');
                let size = elements.len();
                let mut count = 0;

                for (key, element) in elements {
                    count += 1;
                    object_string.push_str(key);
                    object_string.push(':');
                    element.write_to_string_buffer(&mut object_string);
                    if count < size {
                        object_string.push(',');
                    }
                }

                object_string.push('}');
                Value::Object(Cow::Owned(object_string))
            }
            Element::Null(_) => Value::Null,
        }
    }

    fn write_to_string_buffer(&self, buffer: &mut String) {
        match *self {
            Element::String(s)
            | Element::Object(s)
            | Element::Array(s)
            | Element::Boolean(s)
            | Element::Number(s)
            | Element::Null(s) => {
                let s = unsafe { std::str::from_utf8_unchecked(s) };
                buffer.push_str(s);
            }
            Element::List(ref elements) => {
                buffer.push('[');
                let size = elements.len();

                for (i, element) in elements.iter().enumerate() {
                    element.write_to_string_buffer(buffer);
                    if i < size - 1 {
                        buffer.push(',');
                    }
                }

                buffer.push(']');
            }
            _ => (),
        }
    }
}

pub fn true_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    if bytes.len() < 4 {
        return Err(crate::Error::Eof);
    }

    Ok(split_at_u8(bytes, 4))
}

pub fn false_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    if bytes.len() < 5 {
        return Err(crate::Error::Eof);
    }

    Ok(split_at_u8(bytes, 5))
}

pub fn null_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    if bytes.len() < 4 {
        return Err(crate::Error::Eof);
    }

    Ok(split_at_u8(bytes, 4))
}

#[inline(always)]
pub fn split_at(s: &str, mid: usize) -> (&str, &str) {
    unsafe { (s.get_unchecked(..mid), s.get_unchecked(mid..s.len())) }
}

#[inline(always)]
pub fn split_at_u8(s: &[u8], mid: usize) -> (&[u8], &[u8]) {
    unsafe { (s.get_unchecked(..mid), s.get_unchecked(mid..s.len())) }
}

pub fn string_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    // skip check the first byte

    let mut i = 1;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'"' => {
                i += 1;
                break;
            }
            b'\\' => {
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    Ok(split_at_u8(bytes, i))
}

pub fn string(input: &str) -> Result<(&str, &str)> {
    let mut i = 1;
    let bytes = input.as_bytes();
    const CHUNK: usize = 4;

    'outer: while i + CHUNK < bytes.len() {
        for _ in 0..CHUNK {
            let &b = unsafe { bytes.get_unchecked(i) };
            i += 1;
            match b {
                b'"' => {
                    return Ok(split_at(input, i));
                }
                b'\\' => {
                    i += 1;
                    continue 'outer;
                }
                _ => {}
            }
        }
    }

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'"' => {
                i += 1;
                break;
            }
            b'\\' => {
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    return Ok(split_at(input, i));
}

#[cfg(test)]
mod test_string {
    use super::string;

    #[test]
    fn test_string() {
        assert_eq!(
            string(r#""hello": "tom""#),
            Ok((r#""hello""#, r#": "tom""#))
        );

        assert_eq!(string(r#""hello"#), Ok((r#""hello"#, r#""#)));
    }
}

pub fn compound_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    let mut i = 1;
    let mut depth = 1;

    const CHUNK_SIZE: usize = 16;

    const CHAR_TABLE: [u8; 256] = {
        let mut table = [0; 256];
        table['"' as usize] = 1;
        table['[' as usize] = 1;
        table[']' as usize] = 1;
        table['{' as usize] = 1;
        table['}' as usize] = 1;
        table['\\' as usize] = 1;

        table
    };

    'outer: while i + CHUNK_SIZE < bytes.len() {
        for _ in 0..CHUNK_SIZE {
            let &b = unsafe { bytes.get_unchecked(i) };
            if CHAR_TABLE[b as usize] == 0 {
                i += 1;
                continue;
            }

            match b {
                b'"' => {
                    let input = unsafe { bytes.get_unchecked(i..) };
                    let (s, _) = string_u8(input).unwrap();

                    i += s.len();
                    continue 'outer;
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        return Ok(split_at_u8(bytes, i));
                    }
                }
                b'\\' => {
                    i += 2;
                    continue 'outer;
                }
                _ => (),
            }
            i += 1;
        }
    }

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        if CHAR_TABLE[b as usize] == 0 {
            i += 1;
            continue;
        }

        match b {
            b'\\' => {
                i += 1;
            }
            b'"' => {
                let input = unsafe { bytes.get_unchecked(i..) };
                let (s, _) = string_u8(input).unwrap();
                i += s.len();
                continue;
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            _ => (),
        }
        i += 1;
    }

    return Ok(split_at_u8(bytes, i));
}

// object or array
pub fn compound(input: &str) -> Result<(&str, &str)> {
    let bytes = input.as_bytes();
    let mut i = 1;
    let mut depth = 1;

    const CHUNK_SIZE: usize = 32;

    'outer: while i + CHUNK_SIZE < bytes.len() {
        for _ in 0..CHUNK_SIZE {
            let &b = unsafe { bytes.get_unchecked(i) };

            match b {
                b'\\' => {
                    i += 2;
                    continue 'outer;
                }
                b'"' => {
                    let input = unsafe { input.get_unchecked(i..) };
                    let (s, _) = string(input).unwrap();

                    i += s.len();
                    continue 'outer;
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        return Ok(split_at(input, i));
                    }
                }
                _ => (),
            }
            i += 1;
        }
    }

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        match b {
            b'\\' => {
                i += 1;
            }
            b'"' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (s, _) = string(input).unwrap();
                i += s.len();
                continue;
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            _ => (),
        }
        i += 1;
    }

    return Ok(split_at(input, i));
}

#[cfg(test)]
mod test_compound {
    use super::compound;
    use super::Result;

    #[test]
    fn test_compound() -> Result<()> {
        const JSON: &str = r#"{"1":"2", "name": "jack"}xxxx"#;
        let r = compound(JSON)?;

        assert_eq!(r.0, r#"{"1":"2", "name": "jack"}"#);
        assert_eq!(r.1, "xxxx");

        Ok(())
    }
}

pub fn number_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    let mut i = 0;

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'0'..=b'9' => (),
            b'-' | b'.' => (),
            _ => break,
        }
        i += 1;
    }

    Ok(split_at_u8(bytes, i))
}

pub fn read_one(input: &[u8]) -> Result<(Option<Element>, &[u8])> {
    let mut i = 0;

    const TABLE: [u8; 256] = {
        let mut table = [0; 256];
        table[b'"' as usize] = 1;
        table[b't' as usize] = 1;
        table[b'f' as usize] = 1;
        table[b'n' as usize] = 1;
        table[b'{' as usize] = 1;
        table[b'}' as usize] = 1;
        table[b'[' as usize] = 1;
        table[b']' as usize] = 1;
        table[b'0' as usize] = 1;
        table[b'1' as usize] = 1;
        table[b'2' as usize] = 1;
        table[b'3' as usize] = 1;
        table[b'4' as usize] = 1;
        table[b'5' as usize] = 1;
        table[b'6' as usize] = 1;
        table[b'7' as usize] = 1;
        table[b'8' as usize] = 1;
        table[b'9' as usize] = 1;
        table[b'-' as usize] = 1;
        table[b'.' as usize] = 1;

        table
    };

    while i < input.len() {
        let b = unsafe { *input.get_unchecked(i) };
        if TABLE[b as usize] == 0 {
            i += 1;
            continue;
        }

        match b {
            b'"' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = string_u8(input)?;
                return Ok((Some(Element::String(a)), b));
            }
            b't' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = true_u8(input)?;
                return Ok((Some(Element::Boolean(a)), b));
            }
            b'f' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = false_u8(input)?;
                return Ok((Some(Element::Boolean(a)), b));
            }
            b'n' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = null_u8(input)?;
                return Ok((Some(Element::Null(a)), b));
            }
            b'{' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = compound_u8(input)?;
                return Ok((Some(Element::Object(a)), b));
            }
            b'[' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = compound_u8(input)?;
                return Ok((Some(Element::Array(a)), b));
            }
            b'0'..=b'9' | b'-' | b'.' => {
                let input = unsafe { input.get_unchecked(i..) };
                let (a, b) = number_u8(input)?;
                return Ok((Some(Element::Number(a)), b));
            }
            b'}' | b']' => return Ok((None, "".as_bytes())),
            _ => (),
        };
    }
    Ok((None, "".as_bytes()))
}
