/****************************************************************************
*
*   net/tcp.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

// std
use std::mem;
use std::ptr;

// external
use libc;

// internal
use super::endpoint::*;
use super::super::queue;
use super::super::win32;


/****************************************************************************
*
*   Constants
*
***/

const ADDR_BYTES: usize = 16; // size of win32::sockaddr_in
const ADDR_BUFFER_BYTES: usize = ADDR_BYTES + 16;
const ADDRS_BUFFER_BYTES: usize = ADDR_BUFFER_BYTES * 2;


/****************************************************************************
*
*   Client code
*
***/

pub trait TcpReceiveNotify {
    fn on_tcp_receive (
        &mut self,
        data: &[u8],
        conn: &mut TcpConnection
    );
}

pub trait TcpConnectNotify {
    fn on_tcp_connect (&mut self) -> TcpReceiveNotify;
}


/****************************************************************************
*
*   Socket
*
***/

struct Socket {
    handle: queue::Handle,
}

impl Socket {
    //=======================================================================
    pub fn is_valid (&self) -> bool {
        self.handle != win32::INVALID_SOCKET
    }

    //=======================================================================
    pub fn new_invalid () -> Socket {
        Socket {
            handle: win32::INVALID_SOCKET,
        }
    }

    //=======================================================================
    pub fn new () -> Option<Socket> {
        let handle = unsafe { win32::socket(
                win32::AF_INET,
                win32::SOCK_STREAM,
                win32::IPPROTO_TCP
        ) };

        if handle == win32::INVALID_SOCKET {
            return None;
        }

        return Some(Socket {
            handle: handle
        });
    }

    //=======================================================================
    pub fn bind (&mut self, endpoint: Endpoint) -> bool {
        // TODO: support IPv6

        // Create sockaddr for binding
        let Endpoint::V4(ref v4) = endpoint;
        let octets = v4.address().octets();
        let port = ((v4.port() & 0xff) << 8) + ((v4.port() & 0xff00) >> 8);
        let sockaddr = win32::sockaddr_in {
            sin_family: win32::AF_INET as i16,
            sin_port: port,
            sin_addr: win32::in_addr {
                s_b1: octets[0],
                s_b2: octets[1],
                s_b3: octets[2],
                s_b4: octets[3],
            },
            sa_zero: [0; 8]
        };

        // Bind socket to address
        let code = unsafe {
            win32::bind(
                self.handle,
                &sockaddr as *const win32::sockaddr_in,
                mem::size_of::<win32::sockaddr_in>() as i32
            )
        };
        return code == 0;
    }

    //=======================================================================
    pub fn listen (&mut self) -> bool {
        let code = unsafe { win32::listen(self.handle, win32::SOMAXCONN) };
        return code == 0;
    }

    //=======================================================================
    pub fn close (&mut self) {
        if self.handle != win32::INVALID_SOCKET {
            unsafe { win32::closesocket(self.handle) };
            self.handle = win32::INVALID_SOCKET;
        }
    }
}

impl Drop for Socket {
    //=======================================================================
    fn drop (&mut self) {
        self.close();
    }
}


/****************************************************************************
*
*   TcpConnection
*
***/

pub struct TcpConnection {
    handle: queue::Handle,
    local: Endpoint,
    remote: Endpoint,
    notify: Box<TcpReceiveNotify>,
}

impl TcpConnection {
    pub fn handle (&self) -> queue::Handle { self.handle }
    pub fn endpoint_local (&self) -> &Endpoint { &self.local }
    pub fn endpoint_remote (&self) -> &Endpoint { &self.remote }

    //=======================================================================
    pub fn new (
        handle: queue::Handle,
        local: Endpoint,
        remote: Endpoint,
        notify: Box<TcpReceiveNotify>
    ) -> TcpConnection {
        TcpConnection {
            handle: handle,
            local: local,
            remote: remote,
            notify: notify,
        }
    }

    //=======================================================================
    pub fn send (&mut self, data: &[u8]) {
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
    endpoint: Endpoint,

    // Async accept data
    accept: Socket,
    addrs: [u8; ADDRS_BUFFER_BYTES],
}

impl TcpListener {
    pub fn endpoint (&self) -> &Endpoint { &self.endpoint }

    //=======================================================================
    pub fn new (endpoint: Endpoint) -> Option<TcpListener> {
        // Create socket
        let socket = Socket::new();
        if socket.is_none() {
            return None;
        }

        // Bind and listen
        let mut socket = socket.unwrap();
        if !socket.bind(endpoint) || !socket.listen() {
            return None;
        }

        return Some(TcpListener {
            socket: socket,
            endpoint: endpoint,

            accept: Socket::new_invalid(),
            addrs: [0; ADDRS_BUFFER_BYTES],
        });
    }

    //=======================================================================
    pub fn close (&mut self) {
        self.socket.close();
    }

    //=======================================================================
    fn accept (&mut self) -> bool {
        // Proceed only if previous socket was accepted
        if self.accept.is_valid() {
            return true;
        }

        // Get new socket
        let socket = Socket::new();
        if socket.is_none() {
            return false;
        }
        self.accept = socket.unwrap();

        // Reset accept params
        let mut overlapped = win32::OVERLAPPED::new();
        for b in self.addrs.iter_mut() {
            *b = 0;
        }

        // Call accept API
        unsafe {
            win32::AcceptEx(
                self.socket.handle,
                self.accept.handle,
                self.addrs[..ADDRS_BUFFER_BYTES].as_mut_ptr() as *mut libc::c_void,
                0,
                ADDR_BUFFER_BYTES as u32,
                ADDR_BUFFER_BYTES as u32,
                ptr::null_mut(),
                &mut overlapped as *mut win32::OVERLAPPED
            );
        }
        let code = get_error_code();
        return code == (win32::ERROR_IO_PENDING as u32);
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
fn get_error_code () -> u32 {
    return unsafe { win32::WSAGetLastError() };
}
