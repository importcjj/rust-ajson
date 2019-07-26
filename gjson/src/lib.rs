extern crate regex;


mod parser;

mod path;
mod read;
mod wild;
mod util;
mod value;
use parser::Parser;

use path::Path;

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
        println!("get from reader {}", String::from_utf8(vec![b]).unwrap());
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
    // println!("get from object {:?}", path);
    reader.next();

    let mut count = 0;
    while let Some(b) = reader.peek() {
        let v = match b {
            b'"' => {
                let s = reader.read_string();
                Value::String(s)
            }
            b'{' => {
                reader.skip_json();
                Value::Null
                // let raw = reader.read_json();
                // Value::Object(raw, None)
            }
            b'[' => {
                reader.skip_json();
                Value::Null
                // let raw = reader.read_json();
                // Value::Array(raw, None)
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
        // println!("find value {:?}", v);

        count += 1;

        if count % 2 == 1 {
            let key = match v {
                Value::String(s) => s,
                _ => panic!("invalid object key {:?}", v),
            };

            if path.is_match(&key) {
                if path.more {
                    return get_from_reader(reader, path.next);
                }

                return parse_from_reader(reader);
            }
        }
    }
    Value::NotExists
}

fn get_from_array<R>(reader: &mut Parser<R>, path: &Path) -> Value
where
    R: io::Read,
{
    // println!("get from array {:?}", path);
    reader.next();
    
    if !path.arrch {
        return Value::NotExists
    }

    let (idx, idx_get) = match path.part.parse::<usize>() {
        Ok(i) => (i, true),
        Err(_) => (0, false),
    };

    let query = &path.query;
    let mut elements: Vec<Value> = Vec::new();

    let process_query = |v: &Value| -> bool {
        let hit = match v {
            Value::Array(raw, _) | Value::Object(raw, _) => {
                let mut p = Parser::new(raw.as_bytes());
                // let res = get_from_reader(&mut p, query.path);
                
                // query.is_match(&res)
                return true
            }
            _ => {
                if query.path.len() > 0 {
                    false
                } else {
                    query.is_match(v)
                } 
            }
        };

        // println!("does hit? {}", hit);
        hit
    };


    let mut count = 0;

    while let Some(b) = reader.peek() {
        if idx_get && idx == count {
            if path.more {
                return get_from_reader(reader, path.next)
            }

            return parse_from_reader(reader)
        }

        let mut v = match b {
            b'"' => {
                let s = reader.read_string();
                Value::String(s)
            }
            b'{' => {
                let raw = reader.read_json();
                Value::Object(raw, None)
            }
            b'[' => {
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
            b']' => {
                let v = if !query.on && !path.more {
                    Value::Number(count as f64)
                } else if query.on && !query.all {
                    Value::NotExists
                } else if !idx_get {
                    Value::Array("".to_owned(), Some(elements))
                } else {
                    Value::NotExists
                };

                return v
            }
            _ => {
                reader.next();
                continue;
            }
        };

        reader.next();
        count += 1;

        if query.on {
            if !process_query(&v) {
                continue;
            }
        }
                        
        if path.more {
            v = v.get_utf8(path.next)
        }

        if query.on && !query.all {
            return v
        }

        if v.exists() {
            elements.push(v)
        }

    }
    Value::NotExists
}


#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn test_work() {
                let json = r#"
        {
            "name": {"first": "Tom", "last": "Anderson"},
            "age":37,
            "children": ["Sara","Alex","Jack"],
            "fav.movie": "Deer Hunter",
            "friends": [
                {"first": "Dale", "last": "Murphy", "age": 44, "addr": { "street": "a", "num": 1 }},
                {"first": "Roger", "last": "Craig", "age": 68, "addr": { "street": "b", "num": 2 }},
                {"first": "Jane", "last": "Murphy", "age": 47, "addr": { "street": "c", "num": 3 }},
            ]
        }"#;

        println!("{:?}", get_from_str(json, r#"friends.#(first%"D*").addr"#));
        println!("{:?}", get_from_str(json, r#"friends.#(first!%"D*")#.last"#));
        println!("==============");
    }
}