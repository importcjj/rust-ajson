use std::io;

pub trait ByteReader {
    fn position(&self) -> usize;
    fn next(&mut self) -> Option<u8>;
    fn peek(&mut self) -> Option<u8>;
    fn seek(&mut self, new: usize);
    fn slice(&self, start: usize, end: usize) -> &[u8];
}

pub struct RefReader<'a> {
    buffer: &'a [u8],
    offset: usize,
    length: usize,
    max_offset: usize,
}

impl<'a> RefReader<'a> {
    pub fn new(source: &[u8]) -> RefReader {
        RefReader {
            buffer: source,
            offset: 0,
            length: 0,
            max_offset: source.len(),
        }
    }

    // dangerous!!
    pub fn tail<'b>(&self, v: &'b [u8]) -> &'b [u8] {
        &v[self.position()..]
    }

    pub fn slice_for<'b>(&self, v: &'b [u8], end: usize) -> &'b [u8] {
        &v[self.position()..end+1]
    }

    pub fn back(&mut self, offset: usize) {
        if self.position() >= offset {
            let prev = self.position();
            self.seek(prev-offset);
        } 
    }

    pub fn forward(&mut self, offset: usize) {
        if self.position() + offset < self.max_offset {
            let prev = self.position();
            self.seek(prev+offset);
        } 
    }
}



impl<'a> ByteReader for RefReader<'a> {
    fn position(&self) -> usize {
        self.offset
    }
    fn next(&mut self) -> Option<u8> {
        if self.length == 0 || self.length - self.offset == 1 {
            if self.max_offset - self.offset > 1 {
                if self.length > 0 {
                    self.offset += 1;
                }
                self.length += 1;
                return Some(self.buffer[self.offset]);
            }
        }

        if self.length > 0 {
            self.offset += 1;
            return Some(self.buffer[self.offset]);
        }

        None
    }

    fn peek(&mut self) -> Option<u8> {
        if self.length == 0 {
            self.next()
        } else {
            Some(self.buffer[self.offset])
        }
    }

    fn seek(&mut self, new: usize) {
        self.offset = new;
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
        if self.length == 0 || self.length - self.offset == 1 {
            if let Ok(1) = self.source.read(&mut self.byte_buf) {
                if self.length != 0 {
                    self.offset += 1;
                }
                self.length += 1;
                self.buffer.push(self.byte_buf[0]);
                return Some(self.byte_buf[0]);
            }
        }

        if self.length > 0 {
            self.offset += 1;
            return Some(self.buffer[self.offset]);
        }

        None
    }

    fn peek(&mut self) -> Option<u8> {
        if self.length == 0 {
            self.next()
        } else {
            Some(self.buffer[self.offset])
        }
    }

    fn seek(&mut self, new: usize) {
        self.offset = new;
    }

    fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.buffer[start..end + 1]
    }

}
