use path::{Path, Query};
use reader;
use reader::ByteReader;
use std::slice;
use std::str;

pub fn parse_query<'a>(v: &'a [u8]) -> Query<'a> {
    let mut depth = 1;
    let mut reader = reader::RefReader::new(v);
    let mut key_end = 0;
    let mut q = Query::empty();

    reader.forward(2);
    let (key, offset) = parse_path(reader.tail(v));
    reader.forward(offset);

    if key.ok {
        q.set_key(key);
    }


    let op_start = reader.position();
    let mut op_end = op_start;
    while let Some(b) = reader.peek() {
        match b {
            b'!' | b'=' | b'<' | b'>' | b'%'| b' ' => if depth == 1 {
                op_end += 1
            },
            b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => (),
        };
    }
    q
}

pub fn parse_path<'a>(v: &'a [u8]) -> (Path<'a>, usize) {
    let mut current_path = Path::new();
    let mut reader = reader::RefReader::new(v);
    let mut end = 0;

    while let Some(b) = reader.peek() {
        match b {
            b'\\' => {
                reader.next();
            }
            b'!' | b'=' | b'<' | b'>' | b'%' => {
                end = reader.position() - 1;
                break;
            }
            b'.' => {
                end = reader.position() - 1;
                current_path.set_more(true);

                let (next, offset) = parse_path(reader.tail(v));
                current_path.set_next(next);
                reader.forward(offset);
                break;
            }
            b'*' | b'?' => current_path.set_wild(true),
            b'#' => current_path.set_arrch(true),
            b'[' | b'(' => {
                if current_path.arrch {
                    reader.back(2);
                    let query = parse_query(reader.tail(v));
                    current_path.set_q(query);
                }
            }
            _ => (),
        };
    }
    if end == 0 {
        end = reader.position();
        if end == 0 {
            current_path.set_ok(false);
        }
    }
    current_path.set_part(reader.slice_for(v, end));
    (current_path, reader.position())
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
