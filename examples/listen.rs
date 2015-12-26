/****************************************************************************
*
*   examples/listen.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate ioq;

use std::net;


/****************************************************************************
*
*   Main
*
***/

//===========================================================================
fn main () {
    let init = ioq::net::initialize();

    let queue = ioq::Queue::new().unwrap();

    let addr = ioq::net::SocketAddr::new(
        ioq::net::IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)),
        5000
    );
    let listener = ioq::net::TcpListener::new(addr, &queue).unwrap();
    listener.accept().unwrap();

    loop {
        let event = queue.dequeue().unwrap();

        match event {
            ioq::Event::Custom => {},
            ioq::Event::TcpAccept(result) => match result {
                Ok(stream) => {
                    println!("stream: {:?}", stream);
                    listener.accept().unwrap();
                },
                Err(error) => {
                    println!("connect error: {:?}", error);
                    break;
                },
            },
        }
    }

    let _ = init;
}