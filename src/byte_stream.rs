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

#[cfg(test)]
mod tests {
    use std::io::{self, Read, Write};

    use super::ByteStream;

    #[test]
    fn test_simple() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(1024);
        let input_string = b"abcdefgh".repeat(128);

        byte_stream.write(&input_string)?;
        let mut read_buf = [0; 1024];
        byte_stream.read(&mut read_buf)?;
        assert_eq!(input_string, read_buf);

        Ok(())
    }

    #[test]
    fn test_read_truncation() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(1024);
        let input_string = b"abcdefgh".repeat(128);

        byte_stream.write(&input_string)?;
        let mut read_buf = [0; 128];
        byte_stream.read(&mut read_buf)?;
        assert_eq!(input_string[0..128], read_buf);

        Ok(())
    }

    #[test]
    fn test_write_truncation() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(128);
        let input_string = b"abcdefgh".repeat(128);

        byte_stream.write(&input_string)?;
        let mut read_buf = [0; 128];
        byte_stream.read(&mut read_buf)?;
        assert_eq!(input_string[0..128], read_buf);

        Ok(())
    }

    #[test]
    fn test_write_read_multiple_times() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(16);
        byte_stream.write(b"Hello")?;
        byte_stream.write(b" ")?;
        byte_stream.write(b"World")?;

        let mut result = String::new();
        byte_stream.read_to_string(&mut result)?;

        assert_eq!(result, "Hello World");
        Ok(())
    }

    #[test]
    fn test_circular_buffer_behavior() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(8);
        byte_stream.write(b"12345678")?;

        let mut buf = [0; 4];
        byte_stream.read(&mut buf)?;
        assert_eq!(&buf, b"1234");

        byte_stream.write(b"ABCD")?;

        let mut result = String::new();
        byte_stream.read_to_string(&mut result)?;

        assert_eq!(result, "5678ABCD");
        Ok(())
    }

    #[test]
    fn test_partial_reads() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(16);
        byte_stream.write(b"Hello World")?;

        let mut buf = [0; 5];
        byte_stream.read(&mut buf)?;
        assert_eq!(&buf, b"Hello");

        byte_stream.read(&mut buf)?;
        assert_eq!(&buf, b" Worl");

        byte_stream.read(&mut buf)?;
        assert_eq!(&buf[0..1], b"d");

        Ok(())
    }

    #[test]
    fn test_capacity_and_overflow() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(8);
        assert_eq!(byte_stream.capacity(), 8);

        let write_result = byte_stream.write(b"123456789")?;
        assert_eq!(write_result, 8); // Only 8 bytes should be written

        let mut result = String::new();
        byte_stream.read_to_string(&mut result)?;
        assert_eq!(result, "12345678");

        Ok(())
    }

    #[test]
    fn test_empty_reads() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(8);
        let mut buf = [0; 4];
        let read_result = byte_stream.read(&mut buf)?;
        assert_eq!(read_result, 0);
        Ok(())
    }

    #[test]
    fn test_full_writes() -> io::Result<()> {
        let mut byte_stream = ByteStream::new(8);
        byte_stream.write(b"12345678")?;
        let write_result = byte_stream.write(b"9")?;
        assert_eq!(write_result, 0); // No space left, should write 0 bytes
        Ok(())
    }
}
