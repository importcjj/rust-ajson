extern crate regex;


mod parser;

mod path;
mod read;

mod util;
mod value;
use parser::Parser;

use path::Path;
use read::UTF8Reader;

use std::io;
pub use value::Value;

pub fn parse(source: &str) -> Value {
    let mut reader = Parser::new(source.as_bytes());
    parse_from_reader(&mut reader)
}

pub fn parse_from_reader<R>(reader: &mut Parser<R>) -> Value
where
    R: io::Read,
{
    while let Some(b) = reader.peek() {
        let v = match b {
            b'{' => {
                let raw = reader.read_json();
                Value::Object(raw, None)
            }
            b'[' => {
                let raw = reader.read_json();
                Value::Array(raw, None)
            }
            b'"' => {
                let raw = reader.read_string();
                Value::String(raw)
            }
            b'-' | b'0'...b'9' => {
                let f = reader.read_number();
                Value::Number(f)
            }
            b't' => Value::Boolean(true),
            b'f' => Value::Boolean(false),
            b'n' => Value::Null,
            b',' | b':' | b' ' | b'\t' | b'\n' | b'\r' => {
                reader.next();
                continue;
            }
            _ => Value::NotExists,
        };

        return v;
    }
    Value::NotExists
}


pub fn get_from_str(raw: &str, path: &str) -> Value {
    let mut reader = Parser::new(raw.as_bytes());
    get_from_reader(&mut reader, path.as_bytes())
}

fn get_from_reader<R>(reader: &mut Parser<R>, path_u8: &[u8]) -> Value
where
    R: io::Read,
{
    let path = Path::from_utf8(path_u8);

    while let Some(b) = reader.next() {
        match b {
            b'{' => return get_from_object(reader, &path),
            b'[' => return get_from_array(reader, &path),
            _ => continue,
        };
    }
    Value::NotExists
}

fn get_from_object<R>(reader: &mut Parser<R>, path: &Path) -> Value
where
    R: io::Read,
{
    reader.next();
    println!("path {:?}", path);

    let mut count = 0;
    while let Some(b) = reader.peek() {
        let v = match b {
            b'"' => {
                let s = reader.read_string();
                Value::String(s)
            }
            c @ b'{' => {
                let raw = reader.read_json();
                Value::Object(raw, None)
            }
            c @ b'[' => {
                let raw = reader.read_json();
                Value::Array(raw, None)
            }
            b'0'...b'9' | b'-' => {
                let raw = reader.read_number();
                Value::Number(raw)
            }
            b't' => {
                reader.skip(3);
                Value::Boolean(true)
            }
            b'f' => {
                reader.skip(4);
                Value::Boolean(false)
            }
            b'n' => {
                reader.skip(3);
                Value::Null
            }
            _ => {
                reader.next();
                continue;
            }
        };

        reader.next();
        println!("find value {:?}", v);

        count += 1;

        if count % 2 == 1 {
            let key = match v {
                Value::String(s) => s,
                _ => panic!("invalid object key {:?}", v),
            };

            if path.match_part(&key) {
                if path.more {
                    return get_from_reader(reader, path.next);
                }
            
                return parse_from_reader(reader);
            }
        }
    }
    Value::NotExists
}

fn get_from_array<R>(p: &mut Parser<R>, path: &Path) -> Value
where
    R: io::Read,
{
    Value::NotExists
}


fn main() {
    let s = r#"{
  "name": {"\}first\"": "Tom", "last": "Anderson"},
  "age":37,
  "children": ["Sara","Alex","Jack"],
  "fav.movie": "Deer Hunter",
  "friends": [
    {"first": "Dale", "last": "Murphy", "age": 44, "nets": ["ig", "fb", "tw"]},
    {"first": "Roger", "last": "Craig", "age": 68, "nets": ["fb", "tw"]},
    {"first": "Jane", "last": "Murphy", "age": 47, "nets": ["ig", "tw"]}
  ]
}"#;
    let v = get_from_str(s, "name");
    println!("{:?}", v);

    let v = get_from_str(s, "name.last");
    println!("{:?}", v);
}
