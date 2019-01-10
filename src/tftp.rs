//
// tftp.rs
// Copyright (C) 2019 g <g@ABCL>
// Distributed under terms of the MIT license.
//
use super::reader;
use super::writer;

pub type Result<T> = std::result::Result<T, Error>;

// A TFTP Packet
#[derive(Debug)]
pub enum Packet<'a> {
    Read(Filename, TransferMode),
    Write(Filename, TransferMode),
    Data(BlockNum, Data<'a>),
    Ack(BlockNum),
    Error(ErrorCode, ErrorMessage),
}

impl<'a> Packet<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self> {
        let mut buffer = reader::reader::new(&buf);
        println!("{:?}", buffer);
        match Opcode::read(&mut buffer)? {
            Opcode::RRQ => Ok(Packet::Read(
                Filename::read(&mut buffer)?,
                TransferMode::read(&mut buffer)?,
            )),
            Opcode::ACK => Ok(Packet::Ack(BlockNum::read(&mut buffer)?)),
            Opcode::ERROR => Ok(Packet::Error(
                ErrorCode::read(&mut buffer)?,
                ErrorMessage::read(&mut buffer)?,
            )),
            _ => Err(Error::Unknown),
        }
    }

    pub fn opcode(&self) -> Opcode {
        match *self {
            Packet::Read(..) => Opcode::RRQ,
            Packet::Write(..) => Opcode::WRQ,
            Packet::Data(..) => Opcode::DATA,
            Packet::Ack(..) => Opcode::ACK,
            Packet::Error(..) => Opcode::ERROR,
        }
    }

    pub fn write(self, mut buffer: &'a mut [u8]) -> Result<usize> {
        let mut buffer = writer::writer::new(&mut buffer);
        self.opcode().write(&mut buffer)?;
        match self {
            Packet::Read(filename, mode) => {
                filename.write(&mut buffer)?;
                mode.write(&mut buffer)?;
            }
            Packet::Write(filename, mode) => {
                filename.write(&mut buffer)?;
                mode.write(&mut buffer)?;
            }
            Packet::Data(block, data) => {
                block.write(&mut buffer)?;
                data.write(&mut buffer)?;
            }
            Packet::Ack(block) => {
                block.write(&mut buffer)?;
            }
            Packet::Error(code, message) => {
                code.write(&mut buffer)?;
                message.write(&mut buffer)?;
            }
        };
        Ok(buffer.pos)
    }
}
pub enum Opcode {
    RRQ = 1,
    WRQ = 2,
    DATA = 3,
    ACK = 4,
    ERROR = 5,
}

impl Opcode {
    fn read(buffer: &mut reader::reader) -> Result<Self> {
        let code = buffer.take_u16()?;
        println!("Opcode: {}", code);
        match Self::from(code) {
            Some(opcode) => Ok(opcode),
            None => Err(Error::InvalidOpCode(code)),
        }
    }

    pub fn write(self, writer: &mut writer::writer) -> Result<()> {
        writer.put_u16(self as u16)?;
        Ok(())
    }

    fn from(opcode: u16) -> Option<Self> {
        use self::Opcode::*;
        match opcode {
            1 => Some(RRQ),
            2 => Some(WRQ),
            3 => Some(DATA),
            4 => Some(ACK),
            5 => Some(ERROR),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Filename(pub String);
impl Filename {
    fn read(buff: &mut reader::reader) -> Result<Self> {
        Ok(Filename(buff.take_string()?))
    }
    fn write(self, buff: &mut writer::writer) -> Result<()> {
        buff.put_string(&self.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum TransferMode {
    Octet,
}
impl TransferMode {
    fn read(buff: &mut reader::reader) -> Result<Self> {
        let mode = buff.take_string()?;
        match TransferMode::parse(&mode.as_bytes()) {
            Some(txmode) => Ok(txmode),
            None => Err(Error::Unknown),
        }
    }
    fn write(self, buff: &mut writer::writer) -> Result<()> {
        buff.put_string(match self {
            TransferMode::Octet => "octet",
        })?;
        Ok(())
    }
    fn parse(buffer: &[u8]) -> Option<Self> {
        if buffer.eq_ignore_ascii_case("octet".as_bytes()) {
            Some(TransferMode::Octet)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct BlockNum(pub u16);
impl BlockNum {
    fn read(buff: &mut reader::reader) -> Result<Self> {
        Ok(BlockNum(buff.take_u16()?))
    }
    fn write(self, buff: &mut writer::writer) -> Result<()> {
        buff.put_u16(self.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Data<'a>(pub &'a [u8]);
impl<'a> Data<'a> {
    fn read(buff: &mut reader::reader<'a>) -> Result<Self> {
        Ok(Data(buff.take_all()?))
    }
    fn write(self, buff: &mut writer::writer) -> Result<()> {
        buff.put_bytes(&self.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    /// Not defined, see error message (if any).
    NotDefined = 0,
    /// File not found.
    FileNotFound = 1,
    /// Access violation.
    AccessViolation = 2,
    /// Disk full or allocation exceeded.
    DiskFull = 3,
    /// Illegal TFTP operation.
    IllegalOperation = 4,
    /// Unknown transfer ID.
    UnknownTransferId = 5,
    /// File already exists.
    FileAlreadyExists = 6,
    /// No such user.
    NoSuchUser = 7,
    /// Options not acceptable. Defined in RFC-2347.
    BadOptions = 8,
}

impl ErrorCode {
    fn read(buffer: &mut reader::reader) -> Result<Self> {
        let code = buffer.take_u16().unwrap_or(999);
        match Self::from(code) {
            Some(errorcode) => Ok(errorcode),
            None => Err(Error::InvalidErrorCode(code)),
        }
    }

    pub fn write(self, writer: &mut writer::writer) -> Result<()> {
        writer.put_u16(self as u16)?;
        Ok(())
    }

    fn from(code: u16) -> Option<Self> {
        use self::ErrorCode::*;
        match code {
            0 => Some(NotDefined),
            1 => Some(FileNotFound),
            2 => Some(AccessViolation),
            3 => Some(DiskFull),
            4 => Some(IllegalOperation),
            5 => Some(UnknownTransferId),
            6 => Some(FileAlreadyExists),
            7 => Some(NoSuchUser),
            8 => Some(BadOptions),
            _ => None,
        }
    }
}

/// The message in an `ERROR` packet.
#[derive(Debug)]
pub struct ErrorMessage(pub String);

impl ErrorMessage {
    fn read(buffer: &mut reader::reader) -> Result<Self> {
        Ok(ErrorMessage(buffer.take_string()?))
    }

    pub fn write(self, writer: &mut writer::writer) -> Result<()> {
        writer.put_string(&self.0)?;
        Ok(())
    }
}

use std::error;
use std::fmt;
use std::io;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidOpCode(u16),
    InvalidErrorCode(u16),
    ReadError(reader::Error),
    WriteError(writer::Error),
    Unknown,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidOpCode(opcode) => write!(f, "invalid opcode: {}", opcode),
            Error::InvalidErrorCode(errcode) => write!(f, "invalid error code: {}", errcode),
            Error::ReadError(ref error) => write!(f, "packet could not be read: {:?}", error),
            Error::WriteError(ref error) => write!(f, "packet could not be written: {:?}", error),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "tftp packet error"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::ReadError(ref error) => Some(error),
            Error::WriteError(ref error) => Some(error),
            _ => None,
        }
    }
}

impl From<reader::Error> for Error {
    fn from(error: reader::Error) -> Error {
        Error::ReadError(error)
    }
}

impl From<writer::Error> for Error {
    fn from(error: writer::Error) -> Error {
        Error::WriteError(error)
    }
}

impl From<Error> for io::Error {
    fn from(error: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
