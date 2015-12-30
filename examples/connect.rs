/****************************************************************************
*
*   examples/connect.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate ioq;


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

    loop {
        match queue.dequeue().unwrap() {
            ioq::Event::TcpConnect(stream, result) => {
                println!("connect");

                queue.enqueue(Box::new(CustomEvent));
            },

            ioq::Event::Custom => break,

            _ => panic!("Unexpected event"),
        }
    }

    let _ = init;
}