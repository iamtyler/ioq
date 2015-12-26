extern crate ioq;

use std::net;

struct Custom;

impl ioq::Custom for Custom {
    fn execute (self: Box<Self>) {
        println!("custom event");
    }
}

fn main () {
    let init = ioq::net::initialize();

    let queue = ioq::Queue::new().unwrap();
    queue.enqueue(Box::new(Custom)).unwrap();

    let addr = ioq::net::SocketAddr::new(
        ioq::net::IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)),
        5000
    );
    let listener = ioq::net::TcpListener::new(addr, &queue).unwrap();
    listener.accept().unwrap();

    loop {
        match queue.dequeue() {
            Ok(event) => match event {
                ioq::Event::Custom => {},
                ioq::Event::TcpAccept(result) => match result {
                    Ok(stream) => {
                        println!("stream: {:?}", stream);
                        listener.accept().unwrap();
                    },
                    Err(error) => {
                        println!("connect error: {:?}", error);
                        break;
                    }
                },
            },
            Err(error) => {
                println!("queue error: {:?}", error);
                break;
            },
        }
    }

    let _ = init;
}