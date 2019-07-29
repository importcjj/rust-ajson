use path::{Path, Query};
use reader;
use reader::ByteReader;
use std::slice;
use std::str;
use value::Value;

pub fn new_path(v: &[u8]) -> Path {
    let (path, _) = parse_path(v);
    path
}

fn parser_query_value(v: &[u8]) -> (Value, usize) {
    // println!("parse query value {:?}", String::from_utf8_lossy(v));
    let mut reader = reader::RefReader::new(v);
    while let Some(b) = reader.next() {
        let value = match b {
            b't' => {
                reader.read_boolean_value();
                Value::Boolean(true)
            }
            b'f' => {
                reader.read_boolean_value();
                Value::Boolean(false)
            }
            b'n' => {
                reader.read_null_value();
                Value::Null
            }
            b'"' => {
                // println!("======");
                let (start, end) = reader.read_str_value();
                let raw = reader.slice(start + 1, end - 1);
                let s = String::from_utf8_lossy(raw).to_string();
                Value::String(s)
                // Value::Null
            }
            b'0'...b'9' | b'-' => {
                let (start, end) = reader.read_number_value();
                let raw = reader.slice(start, end);
                // TODO
                let f = str::from_utf8(raw).unwrap().parse().unwrap();
                Value::Number(f)
            }
            _ => Value::NotExists,
        };

        return (value, reader.position());
    }

    (Value::NotExists, reader.position())
}

fn parse_query<'a>(v: &'a [u8]) -> (Query<'a>, usize) {
    // println!("parse query {:?}", v);
    // println!("parse query str {:?}", String::from_utf8_lossy(v));

    let mut depth = 1;
    let mut reader = reader::RefReader::new(v);
    let mut q = Query::empty();

    let (key, offset) = parse_path(reader.tail(v));
    // println!("find path in query {:?}, {}", key, offset);
    q.set_key(key);
    reader.forward(offset);


    q.set_on(true);
    let op_start = reader.position();
    let mut op_exist = false;
    let mut op_end = op_start;
    while let Some(b) = reader.peek() {
        match b {
            b'!' | b'=' | b'<' | b'>' | b'%' => {
                if depth == 1 {
                    op_exist = true;
                    op_end = reader.position();
                }
            }
            b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b' ' => continue,
            _ => {
                let (val, offset) = parser_query_value(reader.tail(v));
                q.set_val(val);
                reader.forward(offset);
                break;
            }
        };


        reader.next();
    }

    // println!("op {} {}", op_start, op_end);

    if op_exist {
        let op = String::from_utf8_lossy(reader.slice(op_start, op_end)).to_string();
        q.set_op(op);
    }

    
    match reader.next() {
        Some(b'#') => q.set_all(true),
        Some(_) => reader.back(1),
        None => (),
    }


    (q, reader.position())
}

fn parse_path<'a>(v: &'a [u8]) -> (Path<'a>, usize) {
    // println!("parse path {:?}", String::from_utf8_lossy(v));
    let mut current_path = Path::new();
    let mut reader = reader::RefReader::new(v);
    let mut end = 0;
    let mut part_exsit = true;
    let mut depth = 0;
    current_path.set_ok(true);

    while let Some(b) = reader.peek() {
        match b {
            b'\\' => {
                reader.next();
            }
            b']' | b')' => {
                if depth > 0 {
                    depth -= 0;
                }
                if depth == 0 {
                    end = reader.position() - 1;
                    break;
                }
            }
            b'!' | b'=' | b'<' | b'>' | b'%' => {
                if depth == 0 && reader.position() == 0 {
                    part_exsit = false;
                }
                
                break;
            }
            b'.' => {
                end = reader.position() - 1;
                current_path.set_more(true);
                reader.next();
                let (next, offset) = parse_path(reader.tail(v));
                current_path.set_next(next);
                reader.forward(offset);
                break;
            }
            b'*' | b'?' => current_path.set_wild(true),
            b'#' => current_path.set_arrch(true),
            b'[' | b'(' => {
                depth += 1;
                if depth == 1 && current_path.arrch {
                    reader.next();
                    let (query, offset) = parse_query(reader.tail(v));
                    current_path.set_q(query);
                    reader.forward(offset);
                }
            }
            _ => (),
        };

        end = reader.position();
        reader.next();
    }
    if part_exsit {
        // println!("set path part {}", end);
        current_path.set_part(reader.head(v, end));
    } else {
        current_path.set_ok(false);
    }

    (current_path, reader.position())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fn_parse_path() {
        let v = r#"name"#.as_bytes();
        let p = parse_path(&v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = parse_path(&v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = parse_path(&v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"c?ildren.0"#.as_bytes();
        let p = parse_path(&v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = parse_path(&v);
        println!("{:?}", p);
        println!("======================");
    }

    #[test]
    fn test_fn_parse_query() {
        let v = "first)".as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = "first)#".as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"first="name")"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"nets.#(=="ig"))"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"nets.#(=="ig"))#"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"=="ig")"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"first=)"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"sub_item>7)#.title"#.as_bytes();
        let (q, _) = parse_query(&v);
        println!("{:?}", q);
        println!("======================");
    }
}


pub struct UTF8Reader<'a> {
    pub source: &'a [u8],
    start: bool,
    offset: usize,
    length: usize,
}

impl<'a> UTF8Reader<'a> {
    pub fn new(v: &'a [u8]) -> UTF8Reader<'a> {
        UTF8Reader {
            source: v,
            offset: 0,
            start: false,
            length: v.len(),
        }
    }

    pub fn mark(&self) -> usize {
        self.offset
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn back(&mut self, offset: usize) {
        if self.offset >= offset {
            self.offset -= offset
        }
    }

    pub fn next(&mut self) -> Option<u8> {
        if !self.start {
            self.start = true;
            Some(self.source[self.offset])
        } else if self.offset + 1 < self.length {
            self.offset += 1;
            Some(self.source[self.offset])
        } else {
            None
        }
    }

    pub fn peek(&mut self) -> Option<u8> {
        if self.offset < self.length {
            self.start = true;
            Some(self.source[self.offset])
        } else {
            None
        }
    }

    pub fn slice<'b>(&self, start: usize, end: usize) -> &'b [u8] {
        if start < end && end <= self.offset {
            unsafe { slice::from_raw_parts(self.source[start..end].as_ptr(), end - start) }
        } else {
            &[]
        }
    }

    pub fn head(&self, offset: usize) -> &[u8] {
        if offset <= self.offset {
            &self.source[..offset]
        } else {
            &[]
        }
    }

    pub fn head_contains_last(&self, offset: usize) -> &[u8] {
        if offset <= self.offset {
            &self.source[..offset + 1]
        } else {
            &[]
        }
    }

    pub fn tail(&self, start: usize) -> &[u8] {
        if start < self.offset {
            unsafe {
                let v = &self.source[start..self.offset];
                slice::from_raw_parts(v.as_ptr(), v.len())
            }
        } else {
            &[]
        }
    }

    pub fn tail_contains_last(&self, start: usize) -> &[u8] {
        if start < self.offset {
            &self.source[start..self.offset + 1]
        } else {
            &[]
        }
    }

    pub fn skip_string(&mut self) {
        while let Some(b) = self.next() {
            match b {
                b'"' => break,
                b'\\' => {
                    self.next();
                }
                _ => (),
            };
        }
    }

    // TODO: use macro rule
    pub fn read_string_utf8(&mut self) -> Vec<u8> {
        //  println!("parse str");
        let start = self.mark() + 1;
        self.skip_string();

        let v = self.tail(start);
        v.to_vec()
    }

    pub fn read_string(&mut self) -> String {
        //  println!("parse str");
        let start = self.mark() + 1;
        self.skip_string();

        let v = self.tail(start);
        String::from_utf8_lossy(v).to_string()
    }

    pub fn read_number(&mut self) -> f64 {
        //  println!("parse number");
        let start = self.mark();
        while let Some(b) = self.peek() {
            match b {
                b'0'...b'9' => (),
                b'-' | b'.' => (),
                _ => break,
            };
            self.next();
        }

        let s = str::from_utf8(self.tail(start)).unwrap();
        s.parse().unwrap()
    }

    pub fn read_json(&mut self) -> String {
        //  println!("parse json");
        let start = self.mark();
        let mut depth = 1;
        while let Some(b) = self.next() {
            match b {
                b'\\' => {
                    self.next();
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => (),
            }
        }

        let s = String::from_utf8_lossy(self.tail_contains_last(start)).to_string();
        s
    }
}
