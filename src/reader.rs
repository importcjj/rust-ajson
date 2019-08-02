use std::io;

pub trait ByteReader {
    fn started(&self) -> bool;
    fn position(&self) -> usize;
    fn offset(&self) -> usize;
    fn next(&mut self) -> Option<u8>;
    fn peek(&mut self) -> Option<u8>;
    fn seek(&mut self, new: usize);
    fn slice(&self, start: usize, end: usize) -> &[u8];

    fn back(&mut self, offset: usize) {
        if self.position() >= offset {
            let prev = self.position();
            self.seek(prev - offset);
        } else {
            self.seek(0);
        }
    }

    fn read_boolean_value(&mut self) -> (usize, usize) {
        let i = match self.peek() {
            Some(b't') => 4,
            Some(b'f') => 5,
            _ => panic!("can not find true"),
        };

        let start = self.position();
        for _ in 0..i {
            self.next();
        }

        (start, start + i - 1)
    }

    fn read_null_value(&mut self) -> (usize, usize) {
        match self.peek() {
            Some(b'n') => (),
            _ => panic!("can not find null"),
        };

        let start = self.position();
        for _ in 0..4 {
            self.next();
        }

        (start, start + 3)
    }

    fn read_json_value(&mut self) -> (usize, usize) {
        let _is_object = match self.peek() {
            Some(b'{') => true,
            Some(b'[') => false,
            _ => panic!("Not JSON"),
        };

        let mut depth = 1;
        let start = self.position();

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

        let end = self.position();
        self.next();
        (start, end)
    }

    fn read_str_value(&mut self) -> (usize, usize) {
        let start = self.position();

        let mut ok = false;
        while let Some(b) = self.next() {
            match b {
                b'"' => {
                    ok = true;
                    break;
                }
                b'\\' => {
                    self.next();
                }
                _ => (),
            }
        }
        let mut end = self.position();
        if !ok {
            end += 1;
        }
        self.next();
        (start, end)
    }

    fn read_number_value(&mut self) -> (usize, usize) {
        let start = self.position();
        while let Some(b) = self.next() {
            match b {
                b'0'...b'9' => (),
                b'-' | b'.' => (),
                _ => return (start, self.position() - 1),
            };
        }

        (start, self.position())
    }
}


pub struct RefReader<'a> {
    buffer: &'a [u8],
    offset: usize,
    max_offset: usize,
    overflow: bool,
}

impl<'a> RefReader<'a> {
    pub fn new(source: &[u8]) -> RefReader {
        RefReader {
            buffer: source,
            offset: 0,
            max_offset: source.len(),
            overflow: false,
        }
    }


    // dangerous!!
    pub fn tail<'b>(&self, v: &'b [u8]) -> &'b [u8] {
        if self.overflow {
            return &[];
        }
        &v[self.position()..]
    }

    pub fn head<'b>(&self, v: &'b [u8], end: usize) -> &'b [u8] {
        &v[..end + 1]
    }

    pub fn forward(&mut self, offset: usize) {
        let prev = self.position();
        self.seek(prev + offset);
    }
}


impl<'a> ByteReader for RefReader<'a> {
    fn position(&self) -> usize {
        if !self.started() {
            0
        } else {
            self.offset - 1
        }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn started(&self) -> bool {
        self.offset > 0
    }

    fn next(&mut self) -> Option<u8> {
        if self.overflow {
            return None;
        }

        if self.offset == self.max_offset {
            self.overflow = true;
            return None;
        }

        let b = self.buffer[self.offset];
        self.offset += 1;
        Some(b)
    }

    fn peek(&mut self) -> Option<u8> {
        if self.overflow {
            return None;
        }

        if !self.started() {
            self.next()
        } else {
            Some(self.buffer[self.offset - 1])
        }
    }

    fn seek(&mut self, new: usize) {
        if new < self.max_offset {
            self.offset = new + 1;
            self.overflow = false;
        } else {
            self.offset = self.max_offset;
        }
    }

    fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end + 1]
    }

}

pub struct LazyReader<R>
where
    R: io::Read,
{
    buffer: Vec<u8>,
    source: R,
    length: usize,
    offset: usize,
    byte_buf: [u8; 1],
    overflow: bool,
}

impl<R> LazyReader<R>
where
    R: io::Read,
{
    pub fn new(source: R) -> LazyReader<R> {
        LazyReader {
            buffer: Vec::with_capacity(10000),
            source: source,
            offset: 0,
            length: 0,
            byte_buf: [0; 1],
            overflow: false,
        }
    }
}

impl<R> ByteReader for LazyReader<R>
where
    R: io::Read,
{
    fn started(&self) -> bool {
        self.offset > 0
    }

    fn offset(&self) -> usize {
        self.offset
    }

    fn position(&self) -> usize {
        if !self.started() {
            panic!("reader not started")
        } else {
            self.offset - 1
        }

    }
    fn next(&mut self) -> Option<u8> {
        if self.overflow {
            return None;
        }

        if self.length == self.offset {
            if let Ok(1) = self.source.read(&mut self.byte_buf) {
                self.offset += 1;
                self.length += 1;
                self.buffer.push(self.byte_buf[0]);
                return Some(self.byte_buf[0]);
            } else {
                self.overflow = true;
                return None;
            }
        }

        let b = self.buffer[self.offset];
        self.offset += 1;
        return Some(b);
    }

    fn peek(&mut self) -> Option<u8> {
        if self.overflow {
            return None;
        }

        if !self.started() {
            self.next()
        } else {
            Some(self.buffer[self.offset - 1])
        }
    }

    fn seek(&mut self, new: usize) {
        if new < self.length {
            self.offset = new + 1;
            self.overflow = false;
        } else {
            self.offset = self.length;
        }
    }

    fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end + 1]
    }

}
