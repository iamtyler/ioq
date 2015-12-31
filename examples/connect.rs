/****************************************************************************
*
*   examples/connect.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate ioq;

use std::fs::File;
use std::io::Write;


/****************************************************************************
*
*   CustomEvent
*
***/

struct CustomEvent;

impl ioq::Custom for CustomEvent {
    fn execute (self: Box<Self>) {}
}


/****************************************************************************
*
*   Main
*
***/

//===========================================================================
fn main () {
    // Initialize and create a queue
    let init = ioq::net::initialize();
    let queue = ioq::Queue::new().unwrap();

    // Define remote address (use a google.com address)
    let ip = ioq::net::IpAddr::V4(ioq::net::Ipv4Addr::new(216, 58, 193, 100));
    let remote = ioq::net::SocketAddr::new(ip, 80);

    // Define local address
    let ip = ioq::net::IpAddr::V4(ioq::net::Ipv4Addr::new(0, 0, 0, 0));
    let local = ioq::net::SocketAddr::new(ip, 0);

    // Create stream and connect
    let stream = ioq::net::TcpStream::new(local, queue.clone()).unwrap();
    stream.connect(remote).unwrap();

    // Get document
    let mut bytes: Vec<u8> = Vec::new();
    let mut total = 0;
    const BUFFER_BYTES: usize = 512 * 1024;
    loop {
        match queue.dequeue().unwrap() {
            ioq::Event::TcpConnect(stream, result) => match result {
                Ok(()) => {
                    println!("connect");

                    println!("local: {:?}", stream.addr_local());
                    println!("remote: {:?}", stream.addr_remote());

                    let buffer = String::from("GET /\r\n").into_bytes().into_boxed_slice();
                    stream.send(buffer).unwrap();
                },
                Err(e) => {
                    panic!("connect error: {:?}", e);
                }
            },

            ioq::Event::TcpSend(stream, _, result) => match result {
                Ok(()) => {
                    println!("send");

                    let buffer = Box::new([0u8; BUFFER_BYTES]);
                    stream.receive(buffer).unwrap();
                },
                Err(e) => {
                    panic!("send error: {:?}", e);
                },
            },

            ioq::Event::TcpReceive(stream, buffer, result) => match result {
                Ok(count) => {
                    println!("receive: {}", count);

                    if count == 0 {
                        queue.enqueue(Box::new(CustomEvent)).unwrap();
                        continue;
                    }

                    total += count;
                    bytes.extend(&buffer[..count]);

                    let buffer = Box::new([0u8; BUFFER_BYTES]);
                    stream.receive(buffer).unwrap();
                },
                Err(e) => {
                    panic!("receive error: {:?}", e);
                },
            },

            ioq::Event::Custom => break,

            _ => panic!("Unexpected event"),
        }
    }

    let mut file = File::create("google.html").unwrap();
    file.write(&bytes[..]);

    let _ = init;
}