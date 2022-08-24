use crate::reader::{Bytes, ReaderAction};
use crate::unescape;
use crate::value::Value;
use crate::Number;
use crate::Result;
use std::borrow::Cow;
use std::collections::HashMap;
use std::str;

#[derive(PartialEq, Debug, Clone)]
pub enum Element<'a> {
    String(&'a str),
    Object(&'a str),
    Array(&'a str),
    Null(&'a str),
    Boolean(&'a str),
    Number(&'a str),
    Count(usize),
    List(Vec<Element<'a>>),
    Map(HashMap<&'a str, Element<'a>>),
}

impl<'a> Element<'a> {
    pub fn to_value(&self) -> Value<'a> {
        match &self {
            Element::String(buf) => {
                let s = unescape(buf[1..buf.len() - 1].as_bytes());
                Value::String(Cow::Owned(s))
            }
            Element::Object(s) => Value::Object(Cow::Borrowed(s)),
            Element::Array(s) => Value::Array(Cow::Borrowed(s)),
            Element::Boolean(buf) => match buf.as_bytes()[0] {
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

pub fn read_true<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    for _ in 0..4 {
        bytes.next();
    }

    let s = unsafe { std::str::from_utf8_unchecked(bytes.slice(start, start + 4 - 1)) };
    Ok(Element::Boolean(s))
}

pub fn r#true(input: &str) -> Result<(&str, &str)> {
    if input.len() < 4 {
        return Err(crate::Error::Eof);
    }

    Ok(input.split_at(4))
}

pub fn read_false<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    for _ in 0..5 {
        bytes.next();
    }
    let s = unsafe { std::str::from_utf8_unchecked(bytes.slice(start, start + 4 - 1)) };
    Ok(Element::Boolean(s))
}

pub fn r#false(input: &str) -> Result<(&str, &str)> {
    if input.len() < 5 {
        return Err(crate::Error::Eof);
    }

    Ok(input.split_at(5))
}

pub fn read_null<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    for _ in 0..4 {
        bytes.next();
    }

    let s = unsafe { std::str::from_utf8_unchecked(bytes.slice(start, start + 4 - 1)) };
    Ok(Element::Null(s))
}

pub fn null(input: &str) -> Result<(&str, &str)> {
    if input.len() < 4 {
        return Err(crate::Error::Eof);
    }

    Ok(input.split_at(4))
}

pub fn read_str<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let (start, end) = read_str_range(bytes)?;
    let s = unsafe { std::str::from_utf8_unchecked(bytes.slice(start, start + 4 - 1)) };
    Ok(Element::String(s))
}

pub fn string(input: &str) -> Result<(&str, &str)> {
    // skip check the first byte

    let mut i = 1;
    let bytes = input.as_bytes();
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

    Ok(input.split_at(i))
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

// object or array
pub fn compound(input: &str) -> Result<(&str, &str)> {
    let bytes = input.as_bytes();
    let mut i = 1;
    let mut depth = 1;

    const CHUNK_SIZE: usize = 8;

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
                        return Ok(input.split_at(i));
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

    return Ok(input.split_at(i));
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

pub fn read_str_range(bytes: &mut Bytes) -> Result<(usize, usize)> {
    let start = bytes.position();

    let mut ok = false;
    bytes.next_byte(|reader, b| match b {
        b'"' => {
            ok = true;
            ReaderAction::Break
        }
        b'\\' => ReaderAction::Skip(1),
        _ => ReaderAction::Continue,
    });

    let mut end = bytes.position();
    if !ok {
        end += 1;
    }
    Ok((start, end))
}

pub fn number(input: &str) -> Result<(&str, &str)> {
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'0'..=b'9' => (),
            b'-' | b'.' => (),
            _ => break,
        }
        i += 1;
    }

    Ok(input.split_at(i))
}

#[cfg(test)]
mod test_number {
    use super::number;

    #[test]
    fn test_number() {
        assert_eq!(number("9999,123"), Ok(("9999", ",123")));
        assert_eq!(number("9999"), Ok(("9999", "")));
        assert_eq!(number("-9999,123"), Ok(("-9999", ",123")));
        assert_eq!(number("9999.1112,"), Ok(("9999.1112", ",")));
    }
}

pub fn read_one(mut input: &str) -> Result<(Option<Element>, &str)> {
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        match b {
            b'"' => {
                let (a, b) = string(input)?;
                return Ok((Some(Element::String(a)), b));
            }
            b't' => {
                let (a, b) = r#true(input)?;
                return Ok((Some(Element::Boolean(a)), b));
            }
            b'f' => {
                let (a, b) = r#false(input)?;
                return Ok((Some(Element::Boolean(a)), b));
            }
            b'n' => {
                let (a, b) = null(input)?;
                return Ok((Some(Element::Null(a)), b));
            }
            b'{' => {
                let (a, b) = compound(input)?;
                return Ok((Some(Element::Object(a)), b));
            }
            b'[' => {
                let (a, b) = compound(input)?;
                return Ok((Some(Element::Array(a)), b));
            }
            b'0'..=b'9' | b'-' | b'.' => {
                let (a, b) = number(input)?;
                return Ok((Some(Element::Number(a)), b));
            }
            b'}' | b']' => return Ok((None, "")),
            _ => {
                i += 1;
                input = unsafe { input.get_unchecked(1..) };
                continue;
            }
        };
    }

    Ok((None, ""))
}
