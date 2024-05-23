#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderError {
    UnexpectedEndOfFile,
}

impl std::fmt::Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Unexpected End of File")
    }
}

impl std::error::Error for ReaderError {}

type Result<T> = std::result::Result<T, ReaderError>;

/// A buffer reader, used to marshall data from a byte array into structured
/// data.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    slice: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice }
    }

    pub fn remaining(&self) -> usize {
        self.slice.len()
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.read_slice(n)?;
        Ok(())
    }

    pub fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.slice.len() >= N {
            let array = self.slice[..N].try_into().unwrap();
            self.slice = &self.slice[N..];
            Ok(array)
        } else {
            Err(ReaderError::UnexpectedEndOfFile)
        }
    }

    pub fn read_slice(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.slice.len() >= n {
            let array = &self.slice[..n];
            self.slice = &self.slice[n..];
            Ok(array)
        } else {
            Err(ReaderError::UnexpectedEndOfFile)
        }
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(u8::from_be_bytes(self.read_bytes()?))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.read_bytes()?))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.read_bytes()?))
    }
}
