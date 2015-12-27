/****************************************************************************
*
*   net/tcp.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::mem;
use std::net;
use std::ptr;

use sys;
use queue;
use error::Error;

use super::socket::Socket;
use super::addr::{SocketAddr, IpAddr, AddrFamily};


/****************************************************************************
*
*   Constants
*
***/

const ADDR_BYTES: usize = 16; // size of sys::sockaddr_in
const ADDR_BUFFER_BYTES: usize = ADDR_BYTES + 16;
const ADDRS_BUFFER_BYTES: usize = ADDR_BUFFER_BYTES * 2;


/****************************************************************************
*
*   TcpStream
*
***/

#[derive(Debug)]
pub struct TcpStream {
    socket: Socket,
    local: SocketAddr,
    remote: SocketAddr,
}

impl TcpStream {
    pub fn addr_local (&self) -> &SocketAddr { &self.local }
    pub fn addr_remote (&self) -> &SocketAddr { &self.remote }

    // //=======================================================================
    // fn new (remote: SocketAddr, queue: queue::Queue) -> TcpStream {
    //     TcpStream {
    //         socket: socket,
    //         local: local,
    //         remote: remote,
    //     }
    // }

    //=======================================================================
    pub fn receive (&self, buffer : Box<[u8]>) {
        let _ = buffer;
    }

    //=======================================================================
    pub fn send (&self, data: Box<[u8]>) {
        let _ = data;
    }

    //=======================================================================
    pub fn close (self) {
    }
}


/****************************************************************************
*
*   TcpListener
*
***/

pub struct TcpListener {
    socket: Socket,
    addr: SocketAddr,
}

impl TcpListener {
    pub fn addr (&self) -> SocketAddr { self.addr }

    //=======================================================================
    pub fn new (addr: SocketAddr, queue: &queue::Queue) -> Result<TcpListener, Error> {
        // Create socket
        let socket = Socket::new_from_addr(addr.ip());
        if let Err(error) = socket {
            return Err(error);
        }
        let socket = socket.unwrap();

        // Bind and listen
        if let Err(error) = socket.bind(addr) {
            return Err(error);
        }
        if let Err(error) = socket.listen() {
            return Err(error);
        }

        // Create listener
        let listener = TcpListener {
            socket: socket,
            addr: addr,
        };

        // Associate with queue
        match queue::associate(queue, listener.socket.handle()) {
            Ok(..) => Ok(listener),
            Err(error) => Err(error),
        }
    }

    //=======================================================================
    pub fn accept (&self) -> Result<(), Error> {
        match AcceptContext::new(self.addr.family()) {
            Ok(context) => context.accept(&self.socket),
            Err(error) => Err(error),
        }
    }
}


/****************************************************************************
*
*   AcceptContext
*
***/

struct AcceptContext {
    socket: Socket,
    addrs: [u8; ADDRS_BUFFER_BYTES],
}

impl AcceptContext {
    //=======================================================================
    pub fn new (family: AddrFamily) -> Result<Box<AcceptContext>, Error> {
        let socket = Socket::new_from_family(family);
        if let Err(error) = socket {
            return Err(error);
        }
        let socket = socket.unwrap();

        Ok(Box::new(AcceptContext {
            socket: socket,
            addrs: [0; ADDRS_BUFFER_BYTES],
        }))
    }

    //=======================================================================
    pub fn accept (self: Box<Self>, socket: &Socket) -> Result<(), Error> {
        let raw = Box::into_raw(self);
        let state = Box::new(queue::State::new(unsafe { Box::from_raw(raw) }));
        let context = unsafe { &*raw };//: &Self = unsafe { mem::transmute(raw) };

        let success = unsafe {
            sys::AcceptEx(
                socket.to_raw(),
                context.socket.to_raw(),
                mem::transmute(&context.addrs),
                0,
                ADDR_BUFFER_BYTES as u32,
                ADDR_BUFFER_BYTES as u32,
                ptr::null_mut(),
                state.overlapped_raw()
            )
        };

        if success == 0 {
            let code = Socket::last_error_code();
            if code != sys::ERROR_IO_PENDING {
                return Err(Error::from_os_error_code(code));
            }
        }

        let _ = Box::into_raw(state);
        Ok(())
    }
}

impl queue::Context for AcceptContext {
    //=======================================================================
    fn into_event (self: Box<Self>, _: u32) -> queue::Event {
        // TODO: read addresses

        let stream = TcpStream {
            socket: self.socket,
            local: SocketAddr::new(IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)), 0),
            remote: SocketAddr::new(IpAddr::V4(net::Ipv4Addr::new(0, 0, 0, 0)), 0),
        };

        queue::Event::TcpAccept(Ok(stream))
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> queue::Event {
        let event = queue::Event::TcpAccept(Err(Socket::last_error()));
        event
    }
}


/****************************************************************************
*
*   Tests
*
***/

// TODO: tests