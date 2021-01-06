use core::{
    cmp::min,
    str::{self, Utf8Error},
    usize,
};

#[derive(Debug)]
pub struct LineBuffer {
    buf: [u8; 64],
    pos: usize,
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self {
            buf: [0u8; 64],
            pos: 0,
        }
    }
}

#[allow(dead_code)]
impl LineBuffer {
    pub fn new() -> Self {
        LineBuffer::default()
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    pub fn feed(&mut self, data: &[u8]) -> Result<usize, Utf8Error> {
        str::from_utf8(data)?;
        let copy_len = min(self.buf.len() - self.pos, data.len());
        self.buf[self.pos..(self.pos + copy_len)].copy_from_slice(&data[..copy_len]);
        self.pos += copy_len;
        Ok(copy_len)
    }

    pub fn get_line<'a>(&mut self, s: &'a mut [u8]) -> Result<&'a str, ()> {
        let mut copylen = if self.pos >= self.buf.len() {
            Some(self.buf.len())
        } else {
            None
        };
        for (i, b) in (&self.buf[..self.pos]).iter().enumerate() {
            if *b == b'\n' || *b == b'\r' {
                copylen = Some(i + 1);
                break;
            }
        }

        if let Some(copylen) = copylen {
            &mut s[..copylen].copy_from_slice(&self.buf[..copylen]);
            let linestr = str::from_utf8(&s[..copylen]).map_err(|_| ())?;
            // Remove copied data
            self.buf.copy_within(copylen.., 0);
            self.pos -= copylen;

            Ok(linestr)
        } else {
            Err(())
        }
    }
}
