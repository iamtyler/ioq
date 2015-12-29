/****************************************************************************
*
*   examples/echo.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate ioq;

use std::mem;


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

    // Define listen address
    let ip = ioq::net::IpAddr::V4(ioq::net::Ipv4Addr::new(0, 0, 0, 0));
    let port = 3000;
    let addr = ioq::net::SocketAddr::new(ip, port);

    // Create a TCP listener and schedule accept
    ioq::net::TcpListener::new(addr, queue.clone()).unwrap().accept().unwrap();
    println!("listening at {}", addr);

    // Dequeue events
    loop {
        match queue.dequeue().unwrap() {
            ioq::Event::TcpAccept(_, result) => match result {
                Ok(stream) => {
                    println!("accept");

                    // Schedule receive on new socket
                    if let Err(e) = stream.receive(Box::new([0u8; 1024])) {
                        panic!("receive error: {:?}", e);
                    }

                    // Call listener.accept() here to schedule another accept
                },
                Err(e) => {
                    panic!("TcpAccept error: {:?}", e);
                },
            },

            ioq::Event::TcpReceive(stream, buffer, result) => match result {
                Ok(bytes) => {
                    println!("receive");

                    // A 0-byte receive means the socket was closed
                    if bytes == 0 {
                        continue;
                    }

                    // Treat buffer as text
                    let message: &str = unsafe { mem::transmute(&buffer[..bytes]) };
                    print!("{}", message);

                    // Build HTTP response
                    let buffer = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plan\r\nContent-Length: {}\r\n\r\n{}",
                        bytes,
                        message
                    ).into_bytes().into_boxed_slice();

                    // Send response
                    if let Err(e) = stream.send(buffer) {
                        panic!("send error: {:?}", e);
                    }
                },
                Err(e) => {
                    panic!("TcpReceive error: {:?}", e);
                },
            },

            ioq::Event::TcpSend(_, _, result) => match result {
                Ok(..) => {
                    println!("send");

                    // Enqueue custom event to trigger breaking from this loop
                    if let Err(e) = queue.enqueue(Box::new(CustomEvent)) {
                        panic!("Custom error: {:?}", e);
                    }
                },
                Err(e) => {
                    panic!("TcpReceive error: {:?}", e);
                },
            },

            ioq::Event::Custom => {
                println!("custom");

                break;
            },
        }
    }

    // Ensure init guard survives until the end
    let _ = init;
}