extern crate regex;


mod parser;

mod path;
mod read;
mod token;
mod util;
mod value;
mod wild;

use path::Path;
use parser::Parser;

use std::io;
pub use value::Value;
use token::Token;

pub fn parse(source: &str) -> Value {
    let mut reader = Parser::new(source.as_bytes());
    parse_from_reader(&mut reader).to_value()
}

pub fn parse_from_reader<'a, 'b, R>(reader: &'b mut Parser<R>) -> Token<'a>
where
    R: io::Read,
{
    while let Some(b) = reader.peek() {
        let v = match b {
            b'{' => {
                let raw = reader.read_json_utf8();
                Token::Object(raw, None)
            }
            b'[' => {
                let raw = reader.read_json_utf8();
                Token::Array(raw, None)
            }
            b'"' => {
                let raw = reader.read_string_uf8();
                Token::String(raw)
            }
            b'-' | b'0'...b'9' => {
                let f = reader.read_number_utf8();
                Token::Number(f, None)
            }
            b't' => Token::Boolean(true),
            b'f' => Token::Boolean(false),
            b'n' => Token::Null,
            b',' | b':' | b' ' | b'\t' | b'\n' | b'\r' => {
                reader.next();
                continue;
            }
            _ => Token::NotExists,
        };

        return v;
    }
    Token::NotExists
}


pub fn get_from_str(raw: &str, path: &str) -> Value {
    let mut reader = Parser::new(raw.as_bytes());
    get_from_reader_path(&mut reader, path.as_bytes()).to_value()
}

fn get_from_reader_path<'a, R>(reader: &'a mut Parser<R>, path_u8: &[u8]) -> Token<'a>
where
    R: io::Read,
{
    let path = Path::new_from_utf8(path_u8);
    get_from_reader(reader, &path)
}

fn get_from_reader<'a, 'b, R>(reader: &'b mut Parser<R>, path: &Path) -> Token<'a>
where
    R: io::Read,
{

    while let Some(b) = reader.next() {
        match b {
            b'{' => return get_from_object(reader, &path),
            b'[' => return get_from_array(reader, &path),
            _ => continue,
        };
    }
    Token::NotExists
}

fn get_from_object<'a, 'b, R>(reader: &'b mut Parser<R>, path: &Path) -> Token<'a>
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
            }
            b'[' => {
                reader.skip_json();
                Value::Null
            }
            b'0'...b'9' | b'-' => {
                reader.skip_number();
                Value::Null
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
                    return get_from_reader(reader, path.borrow_next());
                }

                return parse_from_reader(reader);
            }
        }
    }
    Token::NotExists
}

fn get_from_array<'a, 'b, R>(reader: &'b mut Parser<R>, path: &Path) -> Token<'a>
where
    R: io::Read,
{
    // println!("get from array {:?}", path);
    &reader.next();



    let (idx, idx_get) = match path.part.parse::<usize>() {
        Ok(i) => (i, true),
        Err(_) => (0, false),
    };

    if !idx_get && !path.arrch {
        return Token::NotExists;
    }

    let query = &path.query;
    let mut elements: Vec<Value> = Vec::new();

    let process_query = |v: &Token| -> bool {
        let hit = match v {
            Token::Array(ref raw, _) | Token::Object(ref raw, _) => {
                let mut p = Parser::new(*raw);
                let res = get_from_reader_path(&mut p, query.path);

                query.is_match(&res)
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
                return get_from_reader(reader, path.borrow_next());
            }

            return parse_from_reader(reader);
        }

        let mut v = match b {
            b'"' => {
                let s = reader.read_string_uf8();
                Token::String(s)
            }
            b'{' => {
                let raw = reader.read_json_utf8();
                Token::Object(raw, None)
            }
            b'[' => {
                let raw = reader.read_json_utf8();
                Token::Array(raw, None)
            }
            b'0'...b'9' | b'-' => {
                let raw = reader.read_number_utf8();
                Token::Number(raw, None)
            }
            b't' => {
                reader.skip(3);
                Token::Boolean(true)
            }
            b'f' => {
                reader.skip(4);
                Token::Boolean(false)
            }
            b'n' => {
                reader.skip(3);
                Token::Null
            }
            b']' => {
                let v = if !query.on && !path.more {
                    Token::Number(&[], Some(count as f64))
                } else if query.on && !query.all {
                    Token::NotExists
                } else if !idx_get {
                    Token::Array(&[], Some(elements))
                } else {
                    Token::NotExists
                };

                return v;
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
            v = v.get_path(path.borrow_next())
        }

        if query.on && !query.all {
            return v;
        }

        if v.exists() {
            elements.push(v.to_value())
        }

    }
    Token::NotExists
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
        println!(
            "{:?}",
            get_from_str(json, r#"friends.#(first!%"D*")#.last"#)
        );
        println!("==============");
    }
}