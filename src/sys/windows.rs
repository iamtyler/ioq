/****************************************************************************
*
*   sys/windows.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::net;
use std::ptr;

use libc;


/****************************************************************************
*
*   Types
*
***/

pub type HANDLE = *mut libc::c_void;
pub type SOCKET = usize;

pub type BOOL = i32;
pub type ULONG_PTR = *mut u32;
pub type VOID_PTR = *mut libc::c_void;


/****************************************************************************
*
*   Constants
*
***/

const WSADESCRIPTION_LEN: usize = 256;
const WSASYS_STATUS_LEN: usize = 128;

pub const INVALID_SOCKET: SOCKET = !0 as SOCKET;
pub const SOMAXCONN: i32 = 0x7fffffff;

pub const AF_INET: i32 = 2;
pub const AF_INET6: i32 = 23;

pub const SOCK_STREAM: i32 = 1;

pub const IPPROTO_TCP: i32 = 6;

pub const INVALID_HANDLE_VALUE: HANDLE = 0xFFFFFFFFFFFFFFFF as HANDLE;
pub const NULL_HANDLE: HANDLE = 0 as HANDLE;

pub const INFINITE: u32 = 0xFFFFFFFF;

pub const ERROR_IO_PENDING: i32 = 997;


/****************************************************************************
*
*   WSAData
*
***/

#[repr(C)]
pub struct WSAData {
    pub wVersion: u16,
    pub wHighVersion: u16,
    pub szDescription: [u8; WSADESCRIPTION_LEN + 1],
    pub szSystemStatus: [u8; WSASYS_STATUS_LEN + 1],

    // Ignore for v2 and up
    pub iMaxSockets: u16,
    pub iMaxUdpDg: u16,
    pub lpVendorInfo: *mut u8,
}

impl WSAData {
    pub fn new () -> WSAData {
        WSAData {
            wVersion: 0,
            wHighVersion: 0,
            szDescription: [0; WSADESCRIPTION_LEN + 1],
            szSystemStatus: [0; WSASYS_STATUS_LEN + 1],
            iMaxSockets: 0,
            iMaxUdpDg: 0,
            lpVendorInfo: 0 as *mut u8,
        }
    }
}


/****************************************************************************
*
*   sockaddr_in
*
***/

#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: i16,
    pub sin_port: u16,
    pub sin_addr: [u8; 4],
    pub sa_zero: [u8; 8],
}

impl sockaddr_in {
    pub fn new (addr: net::SocketAddrV4) -> sockaddr_in {
        sockaddr_in {
            sin_family: AF_INET as i16,
            sin_port: endian::net_16(addr.port()),
            sin_addr: addr.ip().octets(),
            sa_zero: [0; 8]
        }
    }
}


/****************************************************************************
*
*   sockaddr_in6
*
***/

#[repr(C)]
pub struct sockaddr_in6 {
    pub sin6_family: i16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: [u16; 8],
    pub sin6_scope_id: u32,
}

impl sockaddr_in6 {
    pub fn new (addr: net::SocketAddrV6) -> sockaddr_in6 {
        let segments = addr.ip().segments();
        sockaddr_in6 {
            sin6_family: AF_INET6 as i16,
            sin6_port: endian::net_16(addr.port()),
            sin6_flowinfo: 0, // TODO: proper value
            sin6_addr: [
                endian::net_16(segments[0]),
                endian::net_16(segments[1]),
                endian::net_16(segments[2]),
                endian::net_16(segments[3]),
                endian::net_16(segments[4]),
                endian::net_16(segments[5]),
                endian::net_16(segments[6]),
                endian::net_16(segments[7]),
            ],
            sin6_scope_id: 0, // TODO: proper value
        }
    }
}


/****************************************************************************
*
*   OVERLAPPED
*
***/

#[derive(Clone, Debug)]
#[repr(C)]
pub struct OVERLAPPED {
    pub Internal: ULONG_PTR,
    pub InternalHigh: ULONG_PTR,
    pub Offset: u32,
    pub OffsetHigh: u32,
    pub hEvent: HANDLE,
}

impl OVERLAPPED {
    pub fn new () -> OVERLAPPED {
        OVERLAPPED {
            Internal: ptr::null_mut(),
            InternalHigh: ptr::null_mut(),
            Offset: 0,
            OffsetHigh: 0,
            hEvent: NULL_HANDLE,
        }
    }
}


/****************************************************************************
*
*   Public Functions
*
***/

#[link(name = "kernel32")]
extern "stdcall" {
    pub fn CloseHandle (
        hObject: HANDLE // IN
    ) -> BOOL;

    pub fn CreateIoCompletionPort (
        FileHandle: HANDLE,             // IN
        ExistingCompletionPort: HANDLE, // IN OPT
        CompletionKey: ULONG_PTR,       // IN
        NumberOfConcurrentThreads: u32  // IN
    ) -> HANDLE;

    pub fn GetQueuedCompletionStatus (
        CompletionPort: HANDLE,             // IN
        lpNumberOfBytes: *mut u32,          // OUT
        lpCompletionKey: *mut ULONG_PTR,    // OUT
        lpOverlapped: *mut *mut OVERLAPPED, // OUT
        dwMilliseconds: u32                 // IN
    ) -> BOOL;

    pub fn PostQueuedCompletionStatus (
        CompletionPort: HANDLE,             // IN
        dwNumberOfBytesTransferred: u32,    // IN
        dwCompletionKey: ULONG_PTR,         // IN
        lpOverlapped: *mut OVERLAPPED       // IN OPT
    ) -> BOOL;
}

#[link(name = "Ws2_32")]
extern "stdcall" {
    pub fn bind (
        s: SOCKET,      // IN
        name: VOID_PTR, // IN
        namelen: i32    // IN
    ) -> i32;

    pub fn closesocket (
        s: SOCKET // IN
    ) -> i32;

    pub fn listen (
        s: SOCKET,   // IN
        backlog: i32 // IN
    ) -> i32;

    pub fn socket (
        af: i32,       // IN
        socktype: i32, // IN
        protocol: i32  // IN
    ) -> SOCKET;

    pub fn WSACleanup () -> i32;

    pub fn WSAGetLastError () -> i32;

    pub fn WSAStartup (
        wVersionRequested: u16, // IN
        lpWSAData: *mut WSAData // OUT
    ) -> i32;
}

#[link(name = "wsock32")]
extern "stdcall" {
    pub fn AcceptEx (
        sListenSocket: SOCKET,              // IN
        sAcceptSocket: SOCKET,              // IN
        lpOutputBuffer: VOID_PTR,           // IN
        dwReceveDataLength: u32,            // IN
        dwLocalAddressLength: u32,          // IN
        dwRemoteAddressLength: u32,         // IN
        lpdwBytesReceived: *mut u32,        // OUT
        lpOverlapped: *mut OVERLAPPED       // IN
    ) -> i32;
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
}

#[cfg(target_endian = "big")]
mod endian {
    //=======================================================================
    #[inline]
    pub fn net_16 (n: u16) {
        n
    }
}