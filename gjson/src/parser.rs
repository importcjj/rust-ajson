use std::io;
use std::str;

pub struct Parser<R>
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
    pub fn new(r: R) -> Parser<R> {
        Parser {
            buffer: vec![],
            source: r,
            ch: None,
            length: 0,
            offset: 0,
        }
    }

    pub fn next(&mut self) -> Option<u8> {
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

    pub fn peek(&mut self) -> Option<u8> {
        match self.ch {
            Some(ch) => Some(ch),
            None => self.next(),
        }
    }

    pub fn mark(&self) -> usize {
        self.offset
    }

    pub fn tail(&self, start: usize) -> &[u8] {
        if start < self.offset {
            &self.buffer[start..self.offset]
        } else {
            &[]
        }
    }

    pub fn head(&self, offset: usize) -> &[u8] {
        if offset <= self.offset {
            &self.buffer[..offset]
        } else {
            &[]
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        if start < end && end <= self.offset {
            &self.buffer[start..end]
        } else {
            &[]
        }
    }

    pub fn head_contains_last(&self, offset: usize) -> &[u8] {
        if offset < self.offset {
            &self.buffer[..offset + 1]
        } else {
            &[]
        }
    }

    pub fn tail_contains_last(&self, start: usize) -> &[u8] {
        if start < self.offset {
            &self.buffer[start..self.offset + 1]
        } else {
            &[]
        }
    }

    pub fn read_string(&mut self) -> String {
        // //  println!("parse str");
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


        String::from_utf8_lossy(self.tail(start)).to_string()
    }

    pub fn read_number(&mut self) -> f64 {
        // //  println!("parse number");
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

    pub fn skip(&mut self, i: usize) {
        for _ in 0..i {
            self.next();
        }
    }

    pub fn skip_json(&mut self) {
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
    }

    pub fn read_json(&mut self) -> String {
        // //  println!("parse json");
        let start = self.mark();
        self.skip_json();
        let s = String::from_utf8_lossy(self.tail_contains_last(start)).to_string();
        s
    }
}
