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
    let port = 3000;
    let addr = ioq::net::SocketAddr::new(ip, port);

    // Create a TCP listener and accept
    {
        let listener = ioq::net::TcpListener::new(addr, queue.clone()).unwrap();
        listener.accept().unwrap();
        println!("listening at {}", listener.addr());
    }

    // Dequeue event
    println!("event: {:?}", queue.dequeue().unwrap());

    // Ensure init guard survives until the end
    let _ = init;
}