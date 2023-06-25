pub struct Header {
    chunk_size_with_flags: usize,
}

impl Header {
    pub fn chunk_size(&self) -> usize {
        let flags = self.chunk_size_with_flags & 0xf;
        self.chunk_size_with_flags - flags
    }
}
