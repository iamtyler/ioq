/****************************************************************************
*
*   net/tcp.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::mem;
use std::ptr;
use std::sync::{Arc, Mutex};

use sys;
use queue;
use error::Error;

use super::socket::Socket;
use super::addr::SocketAddr;


/****************************************************************************
*
*   TcpListener
*
***/

#[derive(Debug, Clone)]
pub struct TcpListener {
    inner: Arc<Mutex<TcpListenerInner>>,
}

unsafe impl Sync for TcpListener {}
unsafe impl Send for TcpListener {}

impl TcpListener {
    pub fn addr (&self) -> SocketAddr { self.inner.lock().unwrap().addr }

    //=======================================================================
    pub fn new (addr: SocketAddr, queue: queue::Queue)
        -> Result<TcpListener, Error>
    {
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

        // Associate with queue
        match queue::associate(&queue, socket.handle()) {
            Ok(..) => {
                Ok(TcpListener {
                    inner: Arc::new(Mutex::new(TcpListenerInner {
                        queue: queue,
                        socket: socket,
                        addr: addr,
                    }))
                })
            },
            Err(error) => Err(error),
        }
    }

    //=======================================================================
    pub fn accept (&self) -> Result<(), Error> {
        self.inner.lock().unwrap().accept(self.clone())
    }
}

#[derive(Debug)]
struct TcpListenerInner {
    queue: queue::Queue,
    socket: Socket,
    addr: SocketAddr,
}

impl TcpListenerInner {
    //=======================================================================
    fn accept (&self, listener: TcpListener) -> Result<(), Error> {
        // Create socket
        let socket = Socket::new_from_family(self.addr.family());
        if let Err(error) = socket {
            return Err(error);
        }

        // Create boxed context
        let context = Box::new(TcpAcceptContext {
            listener: listener,
            socket: socket.unwrap(),
            addrs: AddrBuffers::new(),
        });

        // Get raw values from context for passing to OS API
        let socket = context.socket.to_raw();
        let addrs: sys::LPVOID = unsafe { mem::transmute(&context.addrs) };

        // Create boxed state
        let state = Box::new(queue::State::new(context));

        // Call OS API
        let success = unsafe {
            sys::AcceptEx(
                self.socket.to_raw(),
                socket,
                addrs,
                0,
                mem::size_of::<AddrBuffer>() as u32,
                mem::size_of::<AddrBuffer>() as u32,
                ptr::null_mut(),
                state.overlapped_raw()
            ) != 0
        };

        // Handle error
        if !success {
            let code = Socket::last_error_code();
            if code != sys::ERROR_IO_PENDING {
                return Err(Error::from_os_error_code(code));
            }
        }

        // Prevent deallocation of boxed state
        let _ = Box::into_raw(state);
        Ok(())
    }
}


/****************************************************************************
*
*   TcpStream
*
***/

#[derive(Debug, Clone)]
pub struct TcpStream {
    inner: Arc<Mutex<TcpStreamInner>>,
}

unsafe impl Sync for TcpStream {}
unsafe impl Send for TcpStream {}

impl TcpStream {
    pub fn addr_local (&self) -> SocketAddr { self.inner.lock().unwrap().local }
    pub fn addr_remote (&self) -> SocketAddr { self.inner.lock().unwrap().remote }

    //=======================================================================
    pub fn receive (&self, buffer: Box<[u8]>) -> Result<(), Error> {
        self.inner.lock().unwrap().receive(self.clone(), buffer)
    }

    //=======================================================================
    pub fn send (&self, buffer: Box<[u8]>) -> Result<(), Error> {
        self.inner.lock().unwrap().send(self.clone(), buffer)
    }
}

#[derive(Debug)]
pub struct TcpStreamInner {
    socket: Socket,
    local: SocketAddr,
    remote: SocketAddr,
}

impl TcpStreamInner {
    //=======================================================================
    fn receive (&self, stream: TcpStream, mut buffer: Box<[u8]>) -> Result<(), Error> {
        let mut buf = sys::WSABUF::new(&mut buffer[..]);

        let context = Box::new(TcpReceiveContext {
            stream: stream,
            buffer: buffer,
        });

        let state = Box::new(queue::State::new(context));

        let mut flags: u32 = 0;
        let success = unsafe {
            sys::WSARecv(
                self.socket.to_raw(),
                &mut buf as *mut _,
                1,
                ptr::null_mut(),
                &mut flags as *mut _,
                state.overlapped_raw(),
                None
            ) == 0
        };

        if !success {
            let code = Socket::last_error_code();
            if code != sys::ERROR_IO_PENDING {
                return Err(Error::from_os_error_code(code));
            }
        }

        let _ = Box::into_raw(state);
        Ok(())
    }

    //=======================================================================
    fn send (&self, stream: TcpStream, mut buffer: Box<[u8]>) -> Result<(), Error> {
        let mut buf = sys::WSABUF::new(&mut buffer[..]);

        let context = Box::new(TcpSendContext {
            stream: stream,
            buffer: buffer,
        });

        let state = Box::new(queue::State::new(context));

        let flags: u32 = 0;
        let success = unsafe {
            sys::WSASend(
                self.socket.to_raw(),
                &mut buf as *mut _,
                1,
                ptr::null_mut(),
                flags,
                state.overlapped_raw(),
                None
            ) == 0
        };

        if !success {
            let code = Socket::last_error_code();
            if code != sys::ERROR_IO_PENDING {
                return Err(Error::from_os_error_code(code));
            }
        }

        let _ = Box::into_raw(state);
        Ok(())
    }
}


/****************************************************************************
*
*   TcpAcceptContext
*
***/

struct TcpAcceptContext {
    listener: TcpListener,
    socket: Socket,
    addrs: AddrBuffers,
}

impl queue::Context for TcpAcceptContext {
    //=======================================================================
    fn into_event (self: Box<Self>, _: u32) -> queue::Event {
        let result = queue::associate(
            &self.listener.inner.lock().unwrap().queue,
            self.socket.handle()
        );
        match result {
            Err(e) => return queue::Event::TcpAccept(self.listener, Err(e)),
            _ => {},
        }

        let local = self.addrs.local.addr.get_addr().unwrap();
        let remote = self.addrs.remote.addr.get_addr().unwrap();

        let listener = self.listener.clone();
        let stream = TcpStream {
            inner: Arc::new(Mutex::new(TcpStreamInner {
                socket: self.socket,
                local: local,
                remote: remote,
            })),
        };

        queue::Event::TcpAccept(listener, Ok(stream))
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> queue::Event {
        queue::Event::TcpAccept(self.listener, Err(Socket::last_error()))
    }
}


/****************************************************************************
*
*   TcpReceiveContext
*
***/

struct TcpReceiveContext {
    stream: TcpStream,
    buffer: Box<[u8]>,
}

impl queue::Context for TcpReceiveContext {
    //=======================================================================
    fn into_event (self: Box<Self>, bytes: u32) -> queue::Event {
        queue::Event::TcpReceive(
            self.stream.clone(),
            self.buffer,
            Ok(bytes as usize)
        )
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> queue::Event {
        queue::Event::TcpReceive(
            self.stream.clone(),
            self.buffer,
            Err(Socket::last_error())
        )
    }
}


/****************************************************************************
*
*   TcpSendContext
*
***/

struct TcpSendContext {
    stream: TcpStream,
    buffer: Box<[u8]>,
}

impl queue::Context for TcpSendContext {
    //=======================================================================
    fn into_event (self: Box<Self>, _: u32) -> queue::Event {
        queue::Event::TcpSend(
            self.stream.clone(),
            self.buffer,
            Ok(())
        )
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> queue::Event {
        queue::Event::TcpSend(
            self.stream.clone(),
            self.buffer,
            Err(Socket::last_error())
        )
    }
}


/****************************************************************************
*
*   AddrBuffer
*
***/

#[repr(C)]
struct AddrBuffer {
    addr: sys::sockaddr_storage,
    __extra: [u8; sys::SOCKADDR_STORAGE_EXTRA_BYTES],
}

impl AddrBuffer {
    //=======================================================================
    fn new () -> AddrBuffer {
        AddrBuffer {
            addr: sys::sockaddr_storage::new(),
            __extra: [0; sys::SOCKADDR_STORAGE_EXTRA_BYTES],
        }
    }
}


/****************************************************************************
*
*   AddrBuffers
*
***/

#[repr(C)]
struct AddrBuffers {
    local: AddrBuffer,
    remote: AddrBuffer,
}

impl AddrBuffers {
    //=======================================================================
    fn new () -> AddrBuffers {
        AddrBuffers {
            local: AddrBuffer::new(),
            remote: AddrBuffer::new(),
        }
    }
}


/****************************************************************************
*
*   Tests
*
***/

// TODO: tests