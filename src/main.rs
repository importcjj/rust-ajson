use std::cell::RefCell;
use std::io;
use std::str;

struct Parser<R>
where
    R: io::Read,
{
    buffer: Vec<u8>,
    source: R,
    ch: Option<u8>,
    length: usize,
    offset: usize,
}

impl<R> Parser<R>
where
    R: io::Read,
{
    fn new(r: R) -> Parser<R> {
        Parser {
            buffer: vec![],
            source: r,
            ch: None,
            length: 0,
            offset: 0,
        }
    }

    fn next(&mut self) -> Option<u8> {
        let mut c = [0; 1];
        let ch = match self.source.read(&mut c) {
            Ok(1) => {
                self.buffer.push(c[0]);
                self.length += 1;
                if self.ch.is_some() {
                    self.offset += 1;
                }
                Some(c[0])
            }
            _ => None,
        };
        self.ch = ch;
        ch
    }

    fn peek(&mut self) -> Option<u8> {
        match self.ch {
            Some(ch) => Some(ch),
            None => self.next(),
        }
    }

    fn mark(&self) -> usize {
        self.offset
    }

    fn slice(&self, start: usize) -> &[u8] {
        if start < self.offset {
            &self.buffer[start..self.offset]
        } else {
            &[]
        }
    }

    fn slice_contains_last(&self, start: usize) -> &[u8] {
        if start < self.offset {
            &self.buffer[start..self.offset + 1]
        } else {
            &[]
        }
    }

    fn read_string(&mut self) -> String {
        println!("parse str");
        let start = self.mark() + 1;
        while let Some(b) = self.next() {
            match b {
                b'"' => break,
                b'\\' => {
                    self.next();
                }
                _ => (),
            };
        }

        String::from_utf8_lossy(self.slice(start)).to_string()
    }

    fn read_number(&mut self) -> f64 {
        println!("parse number");
        let start = self.mark();
        while let Some(b) = self.peek() {
            match b {
                b'0'...b'9' => (),
                b'-' | b'.' => (),
                _ => break,
            };
            self.next();
        }

        let s = str::from_utf8(self.slice(start)).unwrap();
        s.parse().unwrap()
    }

    fn read_json(&mut self) -> String {
        println!("parse json");
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

        let s = String::from_utf8_lossy(self.slice_contains_last(start)).to_string();
        s
    }
}

pub trait ByteReader {
    fn next(&mut self) -> Option<u8>;
}

fn main() {
    let s = r#"{
  "name\t": {"\}first\"": "Tom", "last": "Anderson"},
  "age":37,
  "children": ["Sara","Alex","Jack"],
  "fav.movie": "Deer Hunter",
  "friends": [
    {"first": "Dale", "last": "Murphy", "age": 44, "nets": ["ig", "fb", "tw"]},
    {"first": "Roger", "last": "Craig", "age": 68, "nets": ["fb", "tw"]},
    {"first": "Jane", "last": "Murphy", "age": 47, "nets": ["ig", "tw"]}
  ]
}"#;
    let mut p = Parser::new(s.as_bytes());
    p.next();
    p.next();
    while let Some(b) = p.peek() {
        match b {
            b'{' | b'[' => {
                let j = p.read_json();
                println!("got json {}", j);
            }
            b'"' => {
                let s = p.read_string();
                println!("got string [{}]", s);
            }
            b'-' | b'0'...b'9' => {
                let n = p.read_number();
                println!("got number [{}]", n);
            }
            _ => (),
        };
        p.next();
    }
}
