pub struct Buffer {
    buf: Vec<u8>,
    ptr: usize,
}

impl From<Buffer> for Vec<u8> {
    fn from(value: Buffer) -> Self {
        value.buf
    }
}

impl From<Vec<u8>> for Buffer {
    fn from(value: Vec<u8>) -> Self {
        Buffer { buf: value, ptr: 0 }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buf: vec![],
            ptr: 0,
        }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Buffer {
            buf: slice.to_vec(),
            ptr: 0,
        }
    }

    pub fn read_u8(&mut self) -> Option<u8> {
        self.ptr += 1;
        self.buf.get(self.ptr - 1).copied()
    }

    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.buf.push(value);
        self
    }

    pub fn read_bool(&mut self) -> Option<bool> {
        self.read_u8().map(|v| v == 1)
    }

    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.write_u8(if value { 1 } else { 0 })
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        let high = self.read_u8()? as u16;
        let low = self.read_u8()? as u16;
        Some(high << 8 | low)
    }

    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.buf.push((value >> 8) as u8);
        self.buf.push((value & 0xFF) as u8);
        self
    }

    pub fn read_js_string(&mut self) -> Option<String> {
        let slice = self.buf.get(self.ptr..)?;
        self.ptr = self.buf.len();
        Some(String::from_utf8_lossy(slice).into_owned())
    }

    pub fn read_string(&mut self) -> Option<String> {
        let len = self.read_u16()? as usize;
        self.ptr += len;
        let slice = self.buf.get((self.ptr - len)..self.ptr)?;
        Some(String::from_utf8_lossy(slice).into_owned())
    }

    pub fn write_string(&mut self, value: &str) -> &mut Self {
        self.write_u16(value.len() as u16);
        self.buf.extend_from_slice(value.as_bytes());
        self
    }
}
