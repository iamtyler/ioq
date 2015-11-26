/**************************************************************************
*
*   win32.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)] // TODO: remove this

use libc;


/**************************************************************************
*
*   WinSock
*
***/

#[cfg(target_pointer_width = "32")]
pub type SOCKET = u32;
#[cfg(target_pointer_width = "64")]
pub type SOCKET = u64;

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

#[repr(C)]
pub struct in_addr {
    pub s_b1: u8,
    pub s_b2: u8,
    pub s_b3: u8,
    pub s_b4: u8,
}

#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: i16,
    pub sin_port: u16,
    pub sin_addr: in_addr,
    pub sa_zero: [u8; 8],
}

#[link(name = "Ws2_32")]
extern "stdcall" {
    pub fn WSAStartup (
        wVersionRequested: u16, // IN
        lpWSAData: *mut WSAData // OUT
    ) -> i32;

    pub fn WSACleanup () -> i32;

    pub fn socket (
        af: i32,       // IN
        socktype: i32, // IN
        protocol: i32  // IN
    ) -> SOCKET;

    pub fn bind (
        s: SOCKET,                // IN
        name: *const sockaddr_in, // IN
        namelen: i32              // IN
    ) -> i32;

    pub fn closesocket (
        s: SOCKET // IN
    ) -> i32;

    pub fn listen (
        s: SOCKET,   // IN
        backlog: i32 // IN
    ) -> i32;

    pub fn accept (
        s: SOCKET,              // IN
        addr: *mut sockaddr_in, // OUT OPT
        addrlen: *mut i32       // IN OUT OPT
    ) -> SOCKET;

    pub fn recv (
        s: SOCKET,    // IN
        buf: *mut u8, // OUT
        len: i32,     // IN
        flags: i32    // IN
    ) -> i32;

    pub fn send (
        s: SOCKET,    // IN
        buf: *mut u8, // IN
        len: i32,     // IN
        flags: i32    // IN
    ) -> i32;

    pub fn WSAGetLastError () -> u32;
}

#[link(name = "wsock32")]
extern "stdcall" {
    pub fn AcceptEx (
        sListenSocket: SOCKET,              // IN
        sAcceptSocket: SOCKET,              // IN
        lpOutputBuffer: *mut libc::c_void,  // IN
        dwReceveDataLength: u32,            // IN
        dwLocalAddressLength: u32,          // IN
        dwRemoteAddressLength: u32,         // IN
        lpdwBytesReceived: *mut u32,        // OUT
        lpOverlapped: *mut OVERLAPPED       // IN
    ) -> i32;
}


/**************************************************************************
*
*   Kernel32
*
***/

pub type HANDLE = *mut libc::c_void;
pub type BOOL = i32;

#[cfg(target_pointer_width = "32")]
pub type ULONG_PTR = u32;
#[cfg(target_pointer_width = "64")]
pub type ULONG_PTR = u64;

pub const INVALID_HANDLE_VALUE: HANDLE = 0xFFFFFFFFFFFFFFFF as HANDLE;
pub const INFINITE: u32 = 0xFFFFFFFF;
pub const NULL_HANDLE: HANDLE = 0 as HANDLE;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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
            Internal: 0,
            InternalHigh: 0,
            Offset: 0,
            OffsetHigh: 0,
            hEvent: NULL_HANDLE,
        }
    }
}

#[link(name = "kernel32")]
extern "stdcall" {
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

    pub fn CloseHandle (
        hObject: HANDLE // IN
    ) -> BOOL;

    pub fn GetLastError () -> u32;
}
