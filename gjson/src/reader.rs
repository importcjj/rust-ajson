use std::io;

pub trait ByteReader {
    fn position(&self) -> usize;
    fn next(&mut self) -> Option<u8>;
    fn peek(&mut self) -> Option<u8>;
    fn seek(&mut self, new: usize);
    fn slice(&self, start: usize, end: usize) -> &[u8];

    fn back(&mut self, offset: usize) {
        if self.position() >= offset {
            let prev = self.position();
            self.seek(prev - offset);
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
        let mut end = start;
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
        end = self.position();
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
                _ => break,
            };
        }

        (start, self.position() - 1)
    }
}


pub struct RefReader<'a> {
    buffer: &'a [u8],
    offset: usize,
    length: usize,
    max_offset: usize,
    read_over: bool,
    started: bool,
}

impl<'a> RefReader<'a> {
    pub fn new(source: &[u8]) -> RefReader {
        RefReader {
            buffer: source,
            offset: 0,
            length: source.len(),
            max_offset: source.len(),
            read_over: source.len() == 0,
            started: false,
        }
    }

    // dangerous!!
    pub fn tail<'b>(&self, v: &'b [u8]) -> &'b [u8] {
        if self.read_over {
            return &[]
        }
        &v[self.position()..]
    }

    pub fn head<'b>(&self, v: &'b [u8], end: usize) -> &'b [u8] {
        &v[..end + 1]
    }

    pub fn forward(&mut self, offset: usize) {
        if self.position() + offset < self.max_offset {
            let prev = self.position();
            self.seek(prev + offset);
        }
    }
}


impl<'a> ByteReader for RefReader<'a> {
    fn position(&self) -> usize {
        self.offset
    }
    fn next(&mut self) -> Option<u8> {
        if self.read_over {
            return None;
        }

        if !self.started {
            self.started = true;
            return Some(self.buffer[0]);
        }

        if self.length - self.offset == 1 {
            self.read_over = true;
            return None;
        }


        self.offset += 1;
        return Some(self.buffer[self.offset]);
    }

    fn peek(&mut self) -> Option<u8> {
        if self.read_over {
            return None;
        }

        if !self.started {
            self.next()
        } else {
            Some(self.buffer[self.offset])
        }
    }

    fn seek(&mut self, new: usize) {
        if new < self.length {
            if self.read_over {
                self.read_over = false;
            }
            self.started = true;
            self.offset = new;
        } else {
            panic!("seek overflow");
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
    read_over: bool,
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
            read_over: false,
        }
    }
}

impl<R> ByteReader for LazyReader<R>
where
    R: io::Read,
{
    fn position(&self) -> usize {
        self.offset
    }
    fn next(&mut self) -> Option<u8> {
        if self.read_over {
            return None;
        }

        if self.length == 0 || self.length - self.offset == 1 {
            if let Ok(1) = self.source.read(&mut self.byte_buf) {
                if self.length != 0 {
                    self.offset += 1;
                }
                self.length += 1;
                self.buffer.push(self.byte_buf[0]);
                return Some(self.byte_buf[0]);
            } else {
                self.read_over = true;
                return None;
            }
        }

        if self.length > 0 {
            self.offset += 1;
            return Some(self.buffer[self.offset]);
        }

        None
    }

    fn peek(&mut self) -> Option<u8> {
        if self.read_over {
            return None;
        }

        if self.length == 0 {
            self.next()
        } else {
            Some(self.buffer[self.offset])
        }
    }

    fn seek(&mut self, new: usize) {
        if self.read_over && new < self.length {
            self.read_over = false;
        }

        self.offset = new;
    }

    fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end + 1]
    }

}
