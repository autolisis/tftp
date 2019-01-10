//
// writer.rs
// Copyright (C) 2019 g <g@ABCL>
// Distributed under terms of the MIT license.
//
extern crate byteorder;
use byteorder::*;

#[derive(Debug, PartialEq)]
pub enum Error {
    NotEnoughSpace,
    StringNotASCII,
    StringContainsNull,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::NotEnoughSpace => write!(f, "not enough space for packet data"),
            Error::StringNotASCII => write!(f, "string is not ASCII"),
            Error::StringContainsNull => write!(f, "string contains null byte"),
        }
    }
}
impl std::error::Error for Error {
    fn description(&self) -> &str {
        "tftp packet write error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct writer<'a> {
    pub buf: &'a mut [u8],
    pub pos: usize,
}

impl<'a> writer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf: buf, pos: 0 }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn rem(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn put_u16(&mut self, value: u16) -> Result<()> {
        if self.rem() >= 2 {
            BigEndian::write_u16(&mut self.buf[self.pos..], value);
            self.pos += 2;
            Ok(())
        } else {
            Err(Error::NotEnoughSpace)
        }
    }

    pub fn put_string(&mut self, value: &str) -> Result<()> {
        if value.is_ascii() {
            if value.contains("\0") {
                Err(Error::StringContainsNull)
            } else {
                let end = self.pos + value.len();
                // Greater-than-or-equals because of the null terminator.
                if end >= self.buf.len() {
                    Err(Error::NotEnoughSpace)
                } else {
                    // TODO: NetASCII nonsense.
                    self.buf[self.pos..end].copy_from_slice(value.as_bytes());
                    self.buf[end] = 0u8;
                    self.pos = end + 1;
                    Ok(())
                }
            }
        } else {
            Err(Error::StringNotASCII)
        }
    }

    pub fn put_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let end = self.pos + bytes.len();
        if end > self.buf.len() {
            Err(Error::NotEnoughSpace)
        } else {
            self.buf[self.pos..end].copy_from_slice(bytes);
            self.pos = end;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
