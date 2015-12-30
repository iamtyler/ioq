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
use std::sync::atomic::ATOMIC_USIZE_INIT;

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
        let socket = Socket::new_from_family(addr.family());
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
        let context = Box::new(AcceptContext {
            queue: self.queue.clone(),
            listener: listener,
            socket: socket.unwrap(),
            addrs: AddrBuffers::new(),
        });

        // Get raw values from context for passing to OS API
        let socket = context.socket.to_raw();
        let addrs: sys::LPVOID = unsafe { mem::transmute(&context.addrs) };

        // Create boxed state
        let state = Box::new(queue::State::new(context));

        // Retrieve OS API
        static ACCEPTEX: sys::WsaExtFn = sys::WsaExtFn {
            guid: sys::WSAID_ACCEPTEX,
            value: ATOMIC_USIZE_INIT,
        };
        let ptr = ACCEPTEX.get(self.socket.to_raw());
        let accept_ex: sys::FN_ACCEPTEX = unsafe { mem::transmute(ptr) };

        // Call OS API
        let success = accept_ex(
            self.socket.to_raw(),
            socket,
            addrs,
            0,
            mem::size_of::<AddrBuffer>() as u32,
            mem::size_of::<AddrBuffer>() as u32,
            ptr::null_mut(),
            state.overlapped_raw()
        ) != 0;

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
    pub fn new (local: SocketAddr, queue: queue::Queue)
        -> Result<TcpStream, Error>
    {
        // Create socket
        let socket = Socket::new_from_family(local.family());
        if let Err(error) = socket {
            return Err(error);
        }
        let socket = socket.unwrap();

        // Bind
        if let Err(error) = socket.bind(local) {
            return Err(error);
        }

        // Associate with queue
        match queue::associate(&queue, socket.handle()) {
            Ok(..) => {
                Ok(TcpStream {
                    inner: Arc::new(Mutex::new(TcpStreamInner {
                        queue: queue,
                        socket: socket,
                        local: local,
                        remote: SocketAddr::new_unspecified(local.family()),
                    }))
                })
            },
            Err(error) => Err(error),
        }
    }

    //=======================================================================
    pub fn connect (self, remote: SocketAddr) -> Result<(), Error> {
        let stream = self.clone();
        self.inner.lock().unwrap().connect(stream, remote)
    }

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
    queue: queue::Queue,
    socket: Socket,
    local: SocketAddr,
    remote: SocketAddr,
}

impl TcpStreamInner {
    //=======================================================================
    pub fn connect (&mut self, stream: TcpStream, remote: SocketAddr) -> Result<(), Error> {
        // Save remote address
        self.remote = remote;

        // Create state
        let state = Box::new(queue::State::new(Box::new(ConnectContext {
            stream: stream
        })));

        // Build sockaddr
        let mut storage = [0u8; sys::SOCKADDR_MAX_BYTES];
        let (sockaddr, len) = Socket::sockaddr_from_addr(remote, &mut storage);

        // Retrieve OS API
        static CONNECTEX: sys::WsaExtFn = sys::WsaExtFn {
            guid: sys::WSAID_CONNECTEX,
            value: ATOMIC_USIZE_INIT,
        };
        let ptr = CONNECTEX.get(self.socket.to_raw());
        let connect_ex: sys::FN_CONNECTEX = unsafe { mem::transmute(ptr) };

        // Call OS API
        let success = connect_ex(
            self.socket.to_raw(),
            sockaddr,
            len,
            ptr::null_mut(),
            0,
            ptr::null_mut(),
            state.overlapped_raw()
        ) != 0;

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

    //=======================================================================
    fn receive (&self, stream: TcpStream, mut buffer: Box<[u8]>) -> Result<(), Error> {
        let mut buf = sys::WSABUF::new(&mut buffer[..]);

        let state = Box::new(queue::State::new(Box::new(ReceiveContext {
            stream: stream,
            buffer: buffer,
        })));

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

        let state = Box::new(queue::State::new(Box::new(SendContext {
            stream: stream,
            buffer: buffer,
        })));

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
*   AcceptContext
*
***/

struct AcceptContext {
    queue: queue::Queue,
    listener: TcpListener,
    socket: Socket,
    addrs: AddrBuffers,
}

impl queue::Context for AcceptContext {
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
        let queue = self.queue.clone();

        let stream = TcpStream {
            inner: Arc::new(Mutex::new(TcpStreamInner {
                queue: queue,
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
*   ConnectContext
*
***/

struct ConnectContext {
    stream: TcpStream,
}

impl queue::Context for ConnectContext {
    //=======================================================================
    fn into_event (self: Box<Self>, _: u32) -> queue::Event {
        // TODO: get local address with getsockname

        queue::Event::TcpConnect(
            self.stream.clone(),
            Ok(())
        )
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> queue::Event {
        queue::Event::TcpConnect(
            self.stream.clone(),
            Err(Socket::last_error())
        )
    }
}


/****************************************************************************
*
*   ReceiveContext
*
***/

struct ReceiveContext {
    stream: TcpStream,
    buffer: Box<[u8]>,
}

impl queue::Context for ReceiveContext {
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
*   SendContext
*
***/

struct SendContext {
    stream: TcpStream,
    buffer: Box<[u8]>,
}

impl queue::Context for SendContext {
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
    extra: [u8; sys::SOCKADDR_STORAGE_EXTRA_BYTES],
}

impl AddrBuffer {
    //=======================================================================
    fn new () -> AddrBuffer {
        AddrBuffer {
            addr: sys::sockaddr_storage::new(),
            extra: [0; sys::SOCKADDR_STORAGE_EXTRA_BYTES],
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