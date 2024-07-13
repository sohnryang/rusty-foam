use std::io::{self, Read, Write};

struct ByteStream {
    capacity: usize,
    head_index: usize,
    used_capacity: usize,
    buffer: Vec<u8>,
}

impl ByteStream {
    pub fn new(capacity: usize) -> Self {
        let buffer = vec![0; capacity];
        Self {
            capacity,
            head_index: 0,
            used_capacity: 0,
            buffer,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Read for ByteStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_size = buf.len().min(self.used_capacity);

        let read_size_lo = read_size.min(self.capacity - self.head_index);
        if read_size_lo > 0 {
            buf[..read_size_lo]
                .copy_from_slice(&self.buffer[self.head_index..self.head_index + read_size_lo]);
        }

        if let Some(read_size_hi) = read_size.checked_sub(read_size_lo) {
            if read_size_hi > 0 {
                buf[read_size_lo..read_size].copy_from_slice(&self.buffer[..read_size_hi]);
            }
        }

        self.used_capacity -= read_size;
        self.head_index = (self.head_index + read_size) % self.capacity;
        Ok(read_size)
    }
}

impl Write for ByteStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let capacity_left = self.capacity - self.used_capacity;
        let write_size = buf.len().min(capacity_left);
        let tail_index = (self.head_index + self.used_capacity) % self.capacity;

        let write_size_lo = write_size.min(self.capacity - tail_index);
        if write_size_lo > 0 {
            let buf_lo = &mut self.buffer[tail_index..tail_index + write_size_lo];
            buf_lo.copy_from_slice(&buf[..write_size_lo]);
        }

        if let Some(write_size_hi) = write_size.checked_sub(write_size_lo) {
            if write_size_hi > 0 {
                let buf_hi = &mut self.buffer[..write_size_hi];
                buf_hi.copy_from_slice(&buf[write_size_lo..]);
            }
        }

        self.used_capacity += write_size;
        Ok(write_size)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
