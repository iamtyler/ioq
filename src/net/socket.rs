/****************************************************************************
*
*   net/socket.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::mem;

use sys;
use error::Error;
use handle::Handle;
use super::addr::{SocketAddr, IpAddr, AddrFamily};


/****************************************************************************
*
*   HandleExt
*
***/

trait HandleExt {
    fn invalid_socket () -> Handle;
    fn from_socket (socket: sys::SOCKET) -> Handle;
    fn to_socket (&self) -> sys::SOCKET;
}

impl HandleExt for Handle {
    fn invalid_socket () -> Handle {
        Handle::from_socket(sys::INVALID_SOCKET)
    }

    fn from_socket (socket: sys::SOCKET) -> Handle {
        Handle::from_raw(socket as sys::HANDLE)
    }

    fn to_socket (&self) -> sys::SOCKET {
        self.to_raw() as sys::SOCKET
    }
}


/****************************************************************************
*
*   Socket
*
***/

#[derive(Debug)]
pub struct Socket {
    handle: Handle,
}

impl Socket {
    pub fn handle (&self) -> Handle { self.handle }
    pub fn to_raw (&self) -> sys::SOCKET { self.handle.to_socket() }

    //=======================================================================
    pub fn is_valid (&self) -> bool {
        self.handle.to_socket() != sys::INVALID_SOCKET
    }

    //=======================================================================
    pub fn new_from_addr (ip: IpAddr) -> Result<Socket, Error> {
        Socket::new_from_family(ip.family())
    }

    //=======================================================================
    pub fn new_from_family (family: AddrFamily) -> Result<Socket, Error> {
        match family {
            AddrFamily::Ipv4 => Socket::new_v4(),
            AddrFamily::Ipv6 => Socket::new_v6(),
        }
    }

    //=======================================================================
    pub fn new_v4 () -> Result<Socket, Error> {
        Socket::new(
            sys::AF_INET,
            sys::SOCK_STREAM,
            sys::IPPROTO_TCP
        )
    }

    //=======================================================================
    pub fn new_v6 () -> Result<Socket, Error> {
        Socket::new(
            sys::AF_INET6,
            sys::SOCK_STREAM,
            sys::IPPROTO_TCP
        )
    }

    //=======================================================================
    pub fn new (af: i32, t: i32, p: i32) -> Result<Socket, Error> {
        let socket = unsafe {
            sys::socket(af, t, p)
        };

        if socket == sys::INVALID_SOCKET {
            Err(Socket::last_error())
        }
        else {
            Ok(Socket {
                handle: Handle::from_raw(socket as sys::HANDLE)
            })
        }
    }

    //=======================================================================
    pub fn bind (&self, addr: SocketAddr) -> Result<(), Error> {
        let sockaddr_in;
        let sockaddr_in6;
        let sockaddr: sys::VOID_PTR;

        match addr {
            SocketAddr::V4(addr) => {
                sockaddr_in = sys::sockaddr_in::new(addr);
                sockaddr = unsafe { mem::transmute(&sockaddr_in) };
            },
            SocketAddr::V6(addr) => {
                sockaddr_in6 = sys::sockaddr_in6::new(addr);
                sockaddr = unsafe { mem::transmute(&sockaddr_in6) };
            }
        }

        // Bind socket to address
        let code = unsafe {
            sys::bind(
                self.handle.to_socket(),
                sockaddr,
                mem::size_of::<sys::sockaddr_in>() as i32
            )
        };

        if code == 0 {
            Ok(())
        }
        else {
            Err(Error::from_os_error_code(code))
        }
    }

    //=======================================================================
    pub fn listen (&self) -> Result<(), Error> {
        let success = unsafe {
            sys::listen(
                self.handle.to_socket(),
                sys::SOMAXCONN
            )
        } == 0;

        if success {
            Ok(())
        }
        else {
            Err(Socket::last_error())
        }
    }

    //=======================================================================
    pub fn close (&mut self) {
        if self.is_valid() {
            unsafe { sys::closesocket(self.handle.to_socket()) };
            self.handle = Handle::invalid_socket();
        }
    }

    //=======================================================================
    pub fn last_error_code () -> i32 {
        unsafe { sys::WSAGetLastError() }
    }

    //=======================================================================
    pub fn last_error () -> Error {
        Error::from_os_error_code(Socket::last_error_code())
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
*   Tests
*
***/

// TODO: tests