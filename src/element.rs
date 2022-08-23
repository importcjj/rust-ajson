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
            Element::Object(buf) => {
                let s = unsafe { str::from_utf8_unchecked(buf) };
                Value::Object(Cow::Borrowed(s))
            }
            Element::Array(buf) => {
                let s = unsafe { str::from_utf8_unchecked(buf) };
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
            Element::String(buf)
            | Element::Object(buf)
            | Element::Array(buf)
            | Element::Boolean(buf)
            | Element::Number(buf)
            | Element::Null(buf) => {
                let s = unsafe { str::from_utf8_unchecked(buf) };
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

    Ok(Element::Boolean(bytes.slice(start, start + 4 - 1)))
}

pub fn read_false<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    for _ in 0..5 {
        bytes.next();
    }

    Ok(Element::Boolean(bytes.slice(start, start + 5 - 1)))
}

pub fn read_null<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    for _ in 0..4 {
        bytes.next();
    }

    Ok(Element::Null(bytes.slice(start, start + 3)))
}

pub fn read_json_range(bytes: &mut Bytes) -> Result<(usize, usize)> {
    let mut depth = 1;
    let start = bytes.position();

    bytes.next_byte(|reader, b| {
        match b {
            b'\\' => return ReaderAction::Skip(1),
            b'"' => {
                let (_, end) = read_str_range(reader).unwrap();
                return ReaderAction::Seek(end);
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    return ReaderAction::Break;
                }
            }
            _ => (),
        }

        ReaderAction::Continue
    });

    let end = bytes.position();
    
    Ok((start, end))
}

pub fn read_object<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let (start, end) = read_json_range(bytes)?;
    Ok(Element::Object(bytes.slice(start, end)))
}

pub fn read_array<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let (start, end) = read_json_range(bytes)?;
    Ok(Element::Array(bytes.slice(start, end)))
}

pub fn read_str<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let (start, end) = read_str_range(bytes)?;
    Ok(Element::String(bytes.slice(start, end)))
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

pub fn read_number<'a>(bytes: &mut Bytes<'a>) -> Result<Element<'a>> {
    let start = bytes.position();
    while let Some(b) = bytes.next() {
        match b {
            b'0'..=b'9' => (),
            b'-' | b'.' => (),
            _ => return Ok(Element::Number(bytes.slice(start, bytes.position() - 1))),
        };
    }

    Ok(Element::Number(bytes.slice(start, bytes.position() - 1)))
}

pub fn read_one<'a>(bytes: &mut Bytes<'a>) -> Result<Option<Element<'a>>> {
    while let Some(b) = bytes.peek() {
        let value = match b {
            b'"' => read_str(bytes)?,
            b't' => read_true(bytes)?,
            b'f' => read_false(bytes)?,
            b'n' => read_null(bytes)?,
            b'{' => read_object(bytes)?,
            b'[' => read_array(bytes)?,
            b'0'..=b'9' | b'-' | b'.' => {
                let n = read_number(bytes)?;
                return Ok(Some(n));
            }
            b'}' | b']' => return Ok(None),
            _ => {
                bytes.next();
                continue;
            }
        };

        bytes.next();

        return Ok(Some(value));
    }

    Ok(None)
}
