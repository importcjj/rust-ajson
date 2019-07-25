use std::str;

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

    pub fn next(&mut self) -> Option<u8> {
        if !self.start && self.offset < self.length {
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

    pub fn slice<'b>(&'b self, start: usize, end: usize) -> &'b [u8] {
        if start < end && end <= self.offset {
            &self.source[start..end]
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
            &self.source[start..self.offset]
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


    // TODO: use macro rule

    pub fn read_string(&mut self) -> String {
        //  println!("parse str");
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