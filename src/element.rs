use std::{borrow::Cow, collections::HashMap};

use crate::{unescape, value::Value, Number, Result};

#[derive(PartialEq, Debug, Clone)]
pub enum Element<'a> {
    String(&'a [u8], bool),
    Object(&'a [u8]),
    Array(&'a [u8]),
    Null(&'a [u8]),
    Boolean(&'a [u8]),
    Number(&'a [u8]),
    Count(usize),
    List(Vec<Element<'a>>),
    Map(HashMap<(&'a [u8], bool), Element<'a>>),
}

impl<'a> Element<'a> {
    pub fn to_value(&self) -> Value<'a> {
        match &self {
            Element::String(buf, esc) => {
                if *esc {
                    let s = unescape(&buf[1..buf.len() - 1]);
                    Value::String(Cow::Owned(s))
                } else {
                    let s = unsafe { std::str::from_utf8_unchecked(&buf[1..buf.len() - 1]) };
                    Value::String(Cow::Borrowed(s))
                }
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

                for ((buf, esc), element) in elements {
                    count += 1;

                    if *esc {
                        let s = unescape(&buf[1..buf.len() - 1]);
                        object_string.push_str(s.as_str());
                    } else {
                        let s = unsafe { std::str::from_utf8_unchecked(buf) };
                        object_string.push_str(s);
                    };

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
            Element::String(buf, esc) => {
                if esc {
                    let s = unescape(&buf[1..buf.len() - 1]);
                    buffer.push_str(s.as_str());
                } else {
                    let s = unsafe { std::str::from_utf8_unchecked(buf) };
                    buffer.push_str(s);
                };
            }

            Element::Object(s)
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
pub fn split_at_u8(s: &[u8], mid: usize) -> (&[u8], &[u8]) {
    unsafe { (s.get_unchecked(..mid), s.get_unchecked(mid..s.len())) }
}

pub fn string_u8(bytes: &[u8]) -> Result<(&[u8], &[u8], bool)> {
    // skip check the first byte

    const TABLE: [u8; 256] = {
        let mut table: [u8; 256] = [0; 256];
        table[b'"' as usize] = 1;
        table[b'\\' as usize] = 1;
        table
    };

    let mut i = 1;
    let mut esc = false;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        if TABLE[b as usize] == 0 {
            i += 1;
            continue;
        }

        match b {
            b'"' => {
                i += 1;
                break;
            }
            b'\\' => {
                esc = true;
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    let (a, b) = split_at_u8(bytes, i);

    Ok((a, b, esc))
}

#[cfg(test)]
mod test_strin_u8 {
    use super::string_u8;

    #[test]
    fn test_string() {
        assert_eq!(
            string_u8(r#""hello": "tom""#.as_bytes()),
            Ok((r#""hello""#.as_bytes(), r#": "tom""#.as_bytes(), false))
        );

        assert_eq!(
            string_u8(r#""hello"#.as_bytes()),
            Ok((r#""hello"#.as_bytes(), r#""#.as_bytes(), false))
        );
    }
}

pub fn compound_u8(bytes: &[u8]) -> Result<(&[u8], &[u8])> {
    fn skip_one(input: &[u8], _: usize) -> Result<usize> {
        Ok(1 + input.len())
    }

    fn skip_string(input: &[u8], i: usize) -> Result<usize> {
        let input = unsafe { input.get_unchecked(i..) };
        let (s, _, _) = string_u8(input)?;
        Ok(s.len())
    }

    fn skip_compound(input: &[u8], i: usize) -> Result<usize> {
        let input = unsafe { input.get_unchecked(i..) };
        let (s, _) = compound_u8(input)?;
        Ok(s.len())
    }

    type Traverse = fn(&[u8], i: usize) -> Result<usize>;
    const TABLE: [Option<Traverse>; 256] = {
        let mut table: [Option<Traverse>; 256] = [None; 256];
        table[b'"' as usize] = Some(skip_string);
        table[b'[' as usize] = Some(skip_compound);
        table[b'{' as usize] = Some(skip_compound);
        table[b']' as usize] = Some(skip_one);
        table[b'}' as usize] = Some(skip_one);
        table
    };

    let mut i = 1;
    const CHUNK: usize = 8;
    'outer: while i + CHUNK < bytes.len() {
        for _ in 0..CHUNK {
            let &b = unsafe { bytes.get_unchecked(i) };
            match TABLE[b as usize] {
                Some(t) => {
                    i += t(bytes, i)?;
                    continue 'outer;
                }
                None => {
                    i += 1;
                }
            }
        }
    }

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        match TABLE[b as usize] {
            Some(t) => {
                i += t(bytes, i)?;
            }
            None => {
                i += 1;
            }
        }
    }

    if i > bytes.len() {
        i -= bytes.len();
    }

    return Ok(split_at_u8(bytes, i));
}

#[cfg(test)]
mod test_compound_u8 {
    use super::{compound_u8, Result};

    #[test]
    fn test_compound() -> Result<()> {
        const JSON: &str = r#"{"1":"2", "name": "jack"}xxxx"#;
        let r = compound_u8(JSON.as_bytes())?;

        assert_eq!(r.0, r#"{"1":"2", "name": "jack"}"#.as_bytes());
        assert_eq!(r.1, "xxxx".as_bytes());

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

pub type MakeResult<'a> = Result<(Option<Element<'a>>, &'a [u8])>;

pub type MakeFn = fn(&[u8]) -> MakeResult;

fn make_string(input: &[u8]) -> MakeResult {
    string_u8(input).map(|(a, b, esc)| (Some(Element::String(a, esc)), b))
}

fn make_true(input: &[u8]) -> MakeResult {
    true_u8(input).map(|(a, b)| (Some(Element::Boolean(a)), b))
}

fn make_false(input: &[u8]) -> MakeResult {
    false_u8(input).map(|(a, b)| (Some(Element::Boolean(a)), b))
}

fn make_null(input: &[u8]) -> MakeResult {
    null_u8(input).map(|(a, b)| (Some(Element::Null(a)), b))
}

fn make_number(input: &[u8]) -> MakeResult {
    number_u8(input).map(|(a, b)| (Some(Element::Number(a)), b))
}

fn make_array(input: &[u8]) -> MakeResult {
    compound_u8(input).map(|(a, b)| (Some(Element::Array(a)), b))
}

fn make_object(input: &[u8]) -> MakeResult {
    compound_u8(input).map(|(a, b)| (Some(Element::Object(a)), b))
}

pub fn read_one(input: &[u8]) -> Result<(Option<Element>, &[u8])> {
    let mut i = 0;

    const MAKER: [Option<MakeFn>; 256] = {
        let mut table: [Option<MakeFn>; 256] = [None; 256];
        table[b'"' as usize] = Some(make_string);
        table[b't' as usize] = Some(make_true);
        table[b'f' as usize] = Some(make_false);
        table[b'n' as usize] = Some(make_null);
        table[b'{' as usize] = Some(make_object);
        table[b'}' as usize] = Some(|input| Ok((None, input)));
        table[b'[' as usize] = Some(make_array);
        table[b']' as usize] = Some(|input| Ok((None, input)));
        table[b'0' as usize] = Some(make_number);
        table[b'1' as usize] = Some(make_number);
        table[b'2' as usize] = Some(make_number);
        table[b'3' as usize] = Some(make_number);
        table[b'4' as usize] = Some(make_number);
        table[b'5' as usize] = Some(make_number);
        table[b'6' as usize] = Some(make_number);
        table[b'7' as usize] = Some(make_number);
        table[b'8' as usize] = Some(make_number);
        table[b'9' as usize] = Some(make_number);
        table[b'-' as usize] = Some(make_number);
        table[b'.' as usize] = Some(make_number);

        table
    };

    while i < input.len() {
        let b = unsafe { *input.get_unchecked(i) };

        match MAKER[b as usize] {
            Some(make_fn) => {
                let input = unsafe { input.get_unchecked(i..) };
                return make_fn(input);
            }
            None => {
                i += 1;
            }
        }
    }
    Ok((None, "".as_bytes()))
}
