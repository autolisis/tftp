//
// tftp.rs
// Copyright (C) 2019 g <g@ABCL>
// Distributed under terms of the MIT license.
//
extern crate byteorder;

use std::net::UdpSocket;

pub fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:34254")?;
    // Receives a single datagram message on the socket. If `buf` is too small to hold
    // the message, it will be cut off.
    let mut buf = [0; 10];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        let buf = &mut buf[..amt];
        println!("{:x?}", buf);
        buf.reverse();
        socket.send_to(buf, &src)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
