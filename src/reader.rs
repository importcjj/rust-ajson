pub struct Bytes<'a> {
    buffer: &'a [u8],
    offset: usize,
    max_offset: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(source: &[u8]) -> Bytes {
        Bytes {
            buffer: source,
            offset: 0,
            max_offset: source.len(),
        }
    }


    #[inline]
    fn overflow(&self) -> bool {
        self.offset == self.max_offset
    }
    

    // dangerous!!
    pub fn tail<'b>(&self, v: &'b [u8]) -> &'b [u8] {
        if self.overflow() {
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

impl<'a> Bytes<'a> {
    pub fn position(&self) -> usize {
        self.offset
    }

    pub fn offset(&self) -> usize {
        self.offset
    }


    pub fn next(&mut self) -> Option<u8> {
        if self.overflow() {
            return None;
        }
        
        self.offset += 1;
        let b =  unsafe { *self.buffer.get_unchecked(self.offset)   };
        
        Some(b)
    }

    pub fn peek(&mut self) -> Option<u8> {
        if self.overflow() {
            return None;
        }

        unsafe { Some(*self.buffer.get_unchecked(self.offset)) }
    }

    pub fn seek(&mut self, new: usize) {
        if new < self.max_offset {
            self.offset = new;
        } else {
            self.offset = self.max_offset;
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        unsafe { self.buffer.get_unchecked(start..end + 1) }
    }

    pub fn back(&mut self, offset: usize) {
        if self.position() >= offset {
            let prev = self.position();
            self.seek(prev - offset);
        } else {
            self.seek(0);
        }
    }
}
