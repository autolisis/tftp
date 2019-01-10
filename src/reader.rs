//
// buf-to-tftp.rs
// Copyright (C) 2019 g <g@ABCL>
// Distributed under terms of the MIT license.
extern crate byteorder;
use byteorder::*;
use std::error;
use std::fmt;

/// A mechanism for progressively reading parts from a packet buffer.
#[derive(Debug)]
pub struct reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    NotEnoughData,
    StringNotTerminated,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotEnoughData => write!(f, "not enough data"),
            Error::StringNotTerminated => write!(f, "string not terminated with null byte"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "tftp packet read error"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<'a> reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf: buf, pos: 0 }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn rem(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn take_u16(&mut self) -> Result<u16> {
        if self.rem() >= 2 {
            let value = BigEndian::read_u16(&self.buf[self.pos..]);
            self.pos += 2;
            Ok(value)
        } else {
            Err(Error::NotEnoughData)
        }
    }

    pub fn take_string(&mut self) -> Result<String> {
        for cur in self.pos..self.buf.len() {
            if self.buf[cur] == 0u8 {
                let bytes = &self.buf[self.pos..cur];
                let string = String::from_utf8_lossy(&bytes);
                self.pos = cur + 1;
                return Ok(string.into_owned());
            }
        }
        Err(Error::StringNotTerminated)
    }

    pub fn take_all(&mut self) -> Result<&'a [u8]> {
        let ref rem = self.buf[self.pos..];
        self.pos = self.buf.len();
        Ok(rem)
    }
}
