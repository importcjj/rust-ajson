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

    pub fn reset(&mut self, input: &'a [u8]) {
        *self = Bytes::new(input)
    }

    pub fn get_buffer(&self) -> &'a [u8] {
        &self.buffer[self.offset..]
    }

    #[inline(always)]
    fn overflow(&self) -> bool {
        self.offset == self.max_offset
    }

    // dangerous!!
    #[inline(always)]
    pub fn tail<'b>(&self, v: &'b [u8]) -> &'b [u8] {
        if self.overflow() {
            return &[];
        }
        &v[self.position()..]
    }

    #[inline(always)]
    pub fn head<'b>(&self, v: &'b [u8], end: usize) -> &'b [u8] {
        &v[..end + 1]
    }

    #[inline(always)]
    pub fn forward(&mut self, offset: usize) {
        let prev = self.position();
        self.seek(prev + offset);
    }
}

pub enum ReaderAction {
    Break,
    Skip(usize),
    Seek(usize),
    Continue,
}

impl<'a> Bytes<'a> {
    #[inline]
    pub fn next_byte<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self, u8) -> ReaderAction,
    {
        'chunk: loop {
            if self.offset + 16 < self.max_offset {
                for _ in 0..16 {
                    self.offset += 1;
                    let b = unsafe { *self.buffer.get_unchecked(self.offset) };
                    match f(self, b) {
                        ReaderAction::Break => return,
                        ReaderAction::Skip(skip_n) => {
                            self.forward(skip_n);
                            continue 'chunk;
                        }
                        ReaderAction::Seek(n) => {
                            self.seek(n);
                            continue 'chunk;
                        }
                        ReaderAction::Continue => (),
                    }
                }
            } else {
                break;
            }
        }

        while let Some(b) = self.next() {
            match f(self, b) {
                ReaderAction::Break => return,
                ReaderAction::Skip(skip_n) => self.forward(skip_n),
                ReaderAction::Seek(n) => self.seek(n),
                ReaderAction::Continue => (),
            }
        }
    }
}

impl<'a> Bytes<'a> {
    #[inline(always)]
    pub fn position(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline(always)]
    pub fn next(&mut self) -> Option<u8> {
        if self.overflow() {
            return None;
        }

        self.offset += 1;
        let b = unsafe { *self.buffer.get_unchecked(self.offset) };

        Some(b)
    }

    #[inline(always)]
    pub fn peek(&mut self) -> Option<u8> {
        if self.overflow() {
            return None;
        }

        unsafe { Some(*self.buffer.get_unchecked(self.offset)) }
    }

    #[inline(always)]
    pub fn seek(&mut self, new: usize) {
        if new < self.max_offset {
            self.offset = new;
        } else {
            self.offset = self.max_offset;
        }
    }

    #[inline(always)]
    pub fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        unsafe { self.buffer.get_unchecked(start..end + 1) }
    }
}
