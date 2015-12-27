/****************************************************************************
*
*   examples/listen.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate ioq;


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
    let port = 5000;
    let addr = ioq::net::SocketAddr::new(ip, port);

    // Create a TCP listener
    let listener;
    match ioq::net::TcpListener::new(addr, &queue) {
        Ok(l) => listener = l,
        Err(e) => panic!("listen error: {}", e),
    }
    println!("listening at {}", listener.addr());

    // Queue a TCP accept event
    listener.accept().unwrap();

    // Dequeue events
    loop {
        let event = queue.dequeue().unwrap();
        match event {
            ioq::Event::TcpAccept(result) => match result {
                Ok(stream) => {
                    println!("stream: {:?}", stream);
                    listener.accept().unwrap();
                },
                Err(error) => {
                    println!("error: {:?}", error);
                    break;
                },
            },
            _ => {
                panic!("Unexpected event");
            },
        }
    }

    // Ensure init guard survives until the end
    let _ = init;
}