use netflow_parser::{NetflowParser, NetflowPacketResult};
use std::net::UdpSocket;

const BUF_SIZE: usize = 2048;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:9005")?;

        loop {
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
            let mut buf = [0; BUF_SIZE];
            let (amt, src) = socket.recv_from(&mut buf)?;

            let mut buf: Vec<u8> = buf.to_vec().iter().filter(|&x| *x != 0).map(|x| x.clone()).collect();
            println!("{:?}", String::from_utf8(buf.clone()));
            // Redeclare `buf` as slice of the received data and send reverse data back to origin.
            let buf = &mut buf[..amt];
            buf.reverse();
            socket.send_to(buf, &src)?;
        }
    } // the socket is closed here
    Ok(())
}
