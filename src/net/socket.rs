/****************************************************************************
*
*   net/socket.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#![allow(dead_code)] // TODO: temporary to prevent compiler warnings

use std::mem;

use error::Error;
use handle::Handle;
use super::addr::*;


/****************************************************************************
*
*   HandleExt
*
***/

pub trait HandleExt {
    fn invalid_socket () -> Handle;
    fn from_socket (socket: os::SOCKET) -> Handle;
    fn to_socket (&self) -> os::SOCKET;
}

impl HandleExt for Handle {
    fn invalid_socket () -> Handle {
        Handle::from_socket(os::INVALID_SOCKET)
    }

    fn from_socket (socket: os::SOCKET) -> Handle {
        Handle::from_raw(socket as os::HANDLE)
    }

    fn to_socket (&self) -> os::SOCKET {
        self.to_raw() as os::SOCKET
    }
}


/****************************************************************************
*
*   Socket
*
***/

pub struct Socket {
    handle: Handle,
}

impl Socket {
    //=======================================================================
    pub fn handle (&self) -> Handle {
        return self.handle;
    }

    //=======================================================================
    pub fn is_valid (&self) -> bool {
        self.handle.to_socket() != os::INVALID_SOCKET
    }

    //=======================================================================
    pub fn new_invalid () -> Socket {
        Socket {
            handle: Handle::from_raw(os::INVALID_SOCKET as os::HANDLE),
        }
    }

    //=======================================================================
    pub fn new_tcp_v4 () -> Result<Socket, Error> {
        Socket::new(
            os::AF_INET,
            os::SOCK_STREAM,
            os::IPPROTO_TCP
        )
    }

    //=======================================================================
    pub fn new_tcp_v6 () -> Result<Socket, Error> {
        Socket::new(
            os::AF_INET6,
            os::SOCK_STREAM,
            os::IPPROTO_TCP
        )
    }

    //=======================================================================
    pub fn new (af: i32, t: i32, p: i32) -> Result<Socket, Error> {
        let socket = unsafe {
            os::socket(af, t, p)
        };

        if socket == os::INVALID_SOCKET {
            Err(Socket::last_error())
        }
        else {
            Ok(Socket {
                handle: Handle::from_raw(socket as os::HANDLE)
            })
        }
    }

    //=======================================================================
    pub fn bind (&self, addr: SocketAddr) -> Result<(), Error> {
        let sockaddr_in;
        let sockaddr_in6;
        let sockaddr: os::VOID_PTR;

        match addr {
            SocketAddr::V4(addr) => {
                sockaddr_in = os::sockaddr_in::new(addr);
                sockaddr = unsafe { mem::transmute(&sockaddr_in) };
            },
            SocketAddr::V6(addr) => {
                sockaddr_in6 = os::sockaddr_in6::new(addr);
                sockaddr = unsafe { mem::transmute(&sockaddr_in6) };
            }
        }

        // Bind socket to address
        let code = unsafe {
            os::bind(
                self.handle.to_socket(),
                sockaddr,
                mem::size_of::<os::sockaddr_in>() as i32
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
        let code = unsafe {
            os::listen(
                self.handle.to_socket(),
                os::SOMAXCONN
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
    pub fn close (&mut self) {
        if self.is_valid() {
            unsafe { os::closesocket(self.handle.to_socket()) };
            self.handle = Handle::invalid_socket();
        }
    }

    //=======================================================================
    pub fn last_error () -> Error {
        let code = unsafe {
            os::WSAGetLastError()
        };

        Error::from_os_error_code(code)
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
*   OS
*
***/

#[cfg(windows)]
mod os {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use std::net;

    use libc;

    use super::endian;


    pub type HANDLE = *mut libc::c_void;
    pub type SOCKET = usize;

    pub type VOID_PTR = *const libc::c_void;

    pub const WSADESCRIPTION_LEN: usize = 256;
    pub const WSASYS_STATUS_LEN: usize = 128;

    pub const INVALID_SOCKET: SOCKET = !0 as SOCKET;
    pub const SOMAXCONN: i32 = 0x7fffffff;
    pub const SOCKET_ERROR: i32 = -1;

    pub const AF_UNSPEC: i32 = 0;
    pub const AF_INET: i32 = 2;
    pub const AF_INET6: i32 = 23;

    pub const SOCK_STREAM: i32 = 1;
    pub const SOCK_DGRAM: i32 = 2;

    pub const IPPROTO_TCP: i32 = 6;
    pub const IPPROTO_UDP: i32 = 17;

    pub const AI_NONE: i32 = 0x00000000;
    pub const AI_PASSIVE: i32 = 0x00000001;

    pub const ERROR_IO_PENDING: i32 = 997;

    #[repr(C)]
    pub struct in_addr {
        octets: [u8; 4],
    }

    impl in_addr {
        pub fn from_octets (octets: [u8; 4]) -> in_addr {
            in_addr {
                octets: octets
            }
        }
    }

    #[repr(C)]
    pub struct sockaddr_in {
        pub sin_family: i16,
        pub sin_port: u16,
        pub sin_addr: in_addr,
        pub sa_zero: [u8; 8],
    }

    impl sockaddr_in {
        pub fn new (addr: net::SocketAddrV4) -> sockaddr_in {
            sockaddr_in {
                sin_family: AF_INET as i16,
                sin_port: endian::net_16(addr.port()),
                sin_addr: in_addr::from_octets(addr.ip().octets()),
                sa_zero: [0; 8]
            }
        }
    }

    #[repr(C)]
    pub struct in6_addr {
        segments: [u16; 8],
    }

    impl in6_addr {
        pub fn from_segments (segments: [u16; 8]) -> in6_addr {
            in6_addr {
                segments: [
                    endian::net_16(segments[0]),
                    endian::net_16(segments[1]),
                    endian::net_16(segments[2]),
                    endian::net_16(segments[3]),
                    endian::net_16(segments[4]),
                    endian::net_16(segments[5]),
                    endian::net_16(segments[6]),
                    endian::net_16(segments[7]),
                ],
            }
        }
    }

    #[repr(C)]
    pub struct sockaddr_in6 {
        pub sin6_family: i16,
        pub sin6_port: u16,
        pub sin6_flowinfo: u32,
        pub sin6_addr: in6_addr,
        pub sin6_scope_id: u32,
    }

    impl sockaddr_in6 {
        pub fn new (addr: net::SocketAddrV6) -> sockaddr_in6 {
            sockaddr_in6 {
                sin6_family: AF_INET6 as i16,
                sin6_port: endian::net_16(addr.port()),
                sin6_flowinfo: 0, // TODO: proper value
                sin6_addr: in6_addr::from_segments(addr.ip().segments()),
                sin6_scope_id: 0, // TODO: proper value
            }
        }
    }

    #[link(name = "Ws2_32")]
    extern "stdcall" {
        pub fn socket (
            af: i32,       // IN
            socktype: i32, // IN
            protocol: i32  // IN
        ) -> SOCKET;

        pub fn bind (
            s: SOCKET,      // IN
            name: VOID_PTR, // IN
            namelen: i32    // IN
        ) -> i32;

        pub fn listen (
            s: SOCKET,   // IN
            backlog: i32 // IN
        ) -> i32;

        pub fn closesocket (
            s: SOCKET // IN
        ) -> i32;

        pub fn WSAGetLastError () -> i32;
    }
}


/****************************************************************************
*
*   Endianness
*
***/

#[cfg(target_endian = "little")]
mod endian {
    //=======================================================================
    #[inline]
    pub fn net_16 (n: u16) -> u16 {
        ((n & 0xff) << 8) + ((n & 0xff00) >> 8)
    }

    //=======================================================================
    #[inline]
    pub fn net_32 (n: u32) -> u32 {
         ((n & 0x000000ff) << 24)
       + ((n & 0x0000ff00) << 8)
       + ((n & 0x00ff0000) >> 8)
       + ((n & 0xff000000) >> 24)
    }
}

#[cfg(target_endian = "big")]
mod endian {
    //=======================================================================
    #[inline]
    pub fn net_16 (n: u16) {
        n
    }

    //=======================================================================
    #[inline]
    pub fn net_32 (n: u32) {
       n
    }
}


/****************************************************************************
*
*   Tests
*
***/

// TODO: tests