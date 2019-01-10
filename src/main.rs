//
// tftp.rs
// Copyright (C) 2019 g <g@ABCL>
// Distributed under terms of the MIT license.
//
extern crate byteorder;

use std::net::*;
mod reader;
mod tftp;
mod writer;

pub fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:69";
    let socket = UdpSocket::bind(addr)?;
    let mut bufin = [0; 512];
    let mut bufout = [0; 4 + 512];
    loop {
        match socket.recv_from(&mut bufin) {
            Ok((size, src)) => match tftp::Packet::parse(&mut bufin[..size]) {
                Ok(packet) => {
                    match handle(addr.parse().unwrap(), src, packet) {
                        Some(packet) => {
                            let size = packet.write(&mut bufout)?;
                            socket.send_to(&bufout[..size], &src)?;
                        }
                        None => {}
                    };
                }
                Err(error) => println!("Ignoring malformed packet {}", error),
            },
            Err(error) => return Err(error),
        }
    }
}

pub fn handle(addr: SocketAddr, src: SocketAddr, packet: tftp::Packet) -> Option<tftp::Packet> {
    println!("Got packet {:?}", packet);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
