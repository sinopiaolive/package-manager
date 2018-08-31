use std::io::{Cursor, Read, Result};

pub struct ProgressIO<IO, F> {
    io: IO,
    total: usize,
    current: usize,
    notify: F,
}

impl<R, F> ProgressIO<R, F>
where
    R: Read,
    F: Fn(usize, usize) -> (),
{
    pub fn reader(total: usize, reader: R, notify: F) -> Self {
        notify(0, total);
        ProgressIO {
            io: reader,
            current: 0,
            total,
            notify,
        }
    }
}

impl<R, F> Read for ProgressIO<R, F>
where
    R: Read,
    F: Fn(usize, usize) -> (),
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.io.read(buf) {
            Ok(size) => {
                self.current += size;
                (self.notify)(self.current, self.total);
                Ok(size)
            }
            e => e,
        }
    }
}

impl<F> ProgressIO<Cursor<Vec<u8>>, F>
where
    F: Fn(usize, usize) -> (),
{
    pub fn reader_from(slice: Vec<u8>, notify: F) -> Self {
        Self::reader(slice.len(), Cursor::new(slice), notify)
    }
}
