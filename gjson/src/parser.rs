use std::io;

use std::slice;
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
            buffer: Vec::with_capacity(1000),
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
                self.length += 1;
                
                if self.ch.is_some() {
                    
                    self.offset += 1;
                }
                self.buffer.push(c[0]);
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

    pub fn tail<'b>(&self, start: usize) -> &'b [u8] {
        if start < self.offset {
            let v = &self.buffer[start..self.offset];
            unsafe { slice::from_raw_parts(v.as_ptr(), v.len()) }
        } else {
            &[]
        }
    }

    pub fn head<'b>(&self, offset: usize) -> &'b [u8] {
        if offset <= self.offset {
            let v = &self.buffer[..offset];
            unsafe { slice::from_raw_parts(v.as_ptr(), v.len()) }
        } else {
            &[]
        }
    }

    pub fn slice<'b>(&self, start: usize, end: usize) -> &'b [u8] {
        if start < end && end <= self.offset {
            let v = &self.buffer[start..end];
            unsafe { slice::from_raw_parts(v.as_ptr(), v.len()) }
        } else {
            &[]
        }
    }

    pub fn head_contains_last<'b>(&self, offset: usize) -> &'b [u8] {
        if offset < self.offset {
            let v = &self.buffer[..offset + 1];
            unsafe { slice::from_raw_parts(v.as_ptr(), v.len()) }
        } else {
            &[]
        }
    }

    pub fn tail_contains_last<'b>(&self, start: usize) -> &'b [u8] {
        if start < self.offset {
            let v = &self.buffer[start..self.offset + 1];
            unsafe { slice::from_raw_parts(v.as_ptr(), v.len()) }
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

    pub fn read_string_uf8<'b>(&mut self) -> Vec<u8> {
        // //  println!("parse str");
        let mut buffer = vec![];
        while let Some(b) = self.next() {
            match b {
                b'"' => break,
                b'\\' => {
                    self.next();
                    buffer.push(b);
                }
                _ => (),
            };
            buffer.push(b);
        }

        // println!("get str {}",String::from_utf8_lossy(&buffer).to_string() );

        buffer


        // self.tail(start)
    }

    pub fn skip_number(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b'0'...b'9' => (),
                b'-' | b'.' => (),
                _ => break,
            };
            self.next();
        }
    }

    pub fn read_number_f64(&mut self) -> f64 {
        // //  println!("parse number");
        let start = self.mark();
        self.skip_number();
        let s = str::from_utf8(self.tail(start)).unwrap();
        s.parse().unwrap()
    }

    pub fn read_number_utf8<'b>(&mut self) -> &'b [u8] {
        let start = self.mark();
        // println!("now at {:?}", self.peek());
        self.skip_number();

        self.tail(start)
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

    pub fn read_json_utf8<'b>(&mut self) -> &'b [u8] {
        // //  println!("parse json");
        let start = self.mark();
        self.skip_json();
        self.tail_contains_last(start)
    }

    pub fn read_json_string(&mut self) -> String {
        // //  println!("parse json");
        let start = self.mark();
        self.skip_json();
        let s = String::from_utf8_lossy(self.tail_contains_last(start)).to_string();
        s
    }
}
