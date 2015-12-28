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

use std::mem;
use std::ptr;

use libc;

use net;


/****************************************************************************
*
*   Types
*
***/

pub type HANDLE = *mut libc::c_void;
pub type SOCKET = usize;
pub type DWORD = u32;
pub type LPDWORD = *mut DWORD;
pub type BOOL = i32;
pub type ULONG_PTR = *mut u32;
pub type LPVOID = *mut libc::c_void;
pub type LPCVOID = *const libc::c_void;
pub type PVOID = *mut libc::c_void;
pub type LPINT = *mut i32;
pub type LPTSTR = *mut u8;
pub type VA_LIST = *mut libc::c_char;
pub type LPWSABUF = *mut WSABUF;

pub type WSAOVERLAPPED_COMPLETION_ROUTINE = extern "C" fn (DWORD, DWORD, *mut OVERLAPPED, DWORD);


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

pub const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;
pub const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x00000200;
pub const FORMAT_MESSAGE_MAX_WIDTH_MASK: u32 = 0x000000FF;

pub const ERROR_INSUFFICIENT_BUFFER: i32 = 122;

pub const SOCKADDR_STORAGE_EXTRA_BYTES: usize = 16;


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
    //=======================================================================
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
#[derive(Debug)]
pub struct sockaddr_in {
    pub sin_family: i16,
    pub sin_port: u16,
    pub sin_addr: [u8; 4],
    pub sa_zero: [u8; 8],
}

impl sockaddr_in {
    //=======================================================================
    pub fn from_addr (addr: net::SocketAddrV4) -> sockaddr_in {
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
#[derive(Debug)]
pub struct sockaddr_in6 {
    pub sin6_family: i16,
    pub sin6_port: u16,
    pub sin6_flowinfo: u32,
    pub sin6_addr: [u16; 8],
    pub sin6_scope_id: u32,
}

impl sockaddr_in6 {
    //=======================================================================
    pub fn from_addr (addr: net::SocketAddrV6) -> sockaddr_in6 {
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
*   sockaddr_storage
*
***/

#[repr(C)]
#[derive(Debug)]
pub struct sockaddr_storage {
    pub ss_family: u16,
    pub __ss_pad1: [u8; 6],
    pub __ss_align: u64,
    pub __ss_pad2: [u8; 16],
    pub __ss_pad3: [u8; 32],
    pub __ss_pad4: [u8; 32],
    pub __ss_pad5: [u8; 32],
}

impl sockaddr_storage {
    //=======================================================================
    pub fn new () -> sockaddr_storage {
        sockaddr_storage {
            ss_family: 0,
            __ss_pad1: [0; 6],
            __ss_align: 0,
            __ss_pad2: [0; 16],
            __ss_pad3: [0; 32],
            __ss_pad4: [0; 32],
            __ss_pad5: [0; 32],
        }
    }

    //=======================================================================
    pub fn get_addr (&self) -> Option<net::SocketAddr> {
        match self.ss_family as i32 {
            AF_INET => {
                let addr: &sockaddr_in = unsafe { mem::transmute(self) };
                let octets = addr.sin_addr;
                let ip = net::Ipv4Addr::new(
                    octets[0],
                    octets[1],
                    octets[2],
                    octets[3]
                );
                let port = endian::net_16(addr.sin_port);

                Some(net::SocketAddr::new(net::IpAddr::V4(ip), port))
            },
            AF_INET6 => {
                let addr: &sockaddr_in6 = unsafe { mem::transmute(self) };
                let segments = addr.sin6_addr;
                let ip = net::Ipv6Addr::new(
                    segments[0],
                    segments[1],
                    segments[2],
                    segments[3],
                    segments[4],
                    segments[5],
                    segments[6],
                    segments[7]
                );
                let port = endian::net_16(addr.sin6_port);

                Some(net::SocketAddr::new(net::IpAddr::V6(ip), port))
            },
            _ => None,
        }
    }
}


/****************************************************************************
*
*   OVERLAPPED
*
***/

#[repr(C)]
#[derive(Clone, Debug)]
pub struct OVERLAPPED {
    pub Internal: ULONG_PTR,
    pub InternalHigh: ULONG_PTR,
    pub Offset: u32,
    pub OffsetHigh: u32,
    pub hEvent: HANDLE,
}

impl OVERLAPPED {
    //=======================================================================
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
*   WSABUF
*
***/

#[allow(dead_code)] // TODO: remove
#[repr(C)]
pub struct WSABUF {
    pub len: u32,
    pub buf: *mut u8,
}

impl WSABUF {
    //=======================================================================
    #[allow(dead_code)] // TODO: remove
    pub fn new (buffer: &mut [u8]) -> WSABUF {
        WSABUF {
            len: buffer.len() as u32,
            buf: buffer.as_mut_ptr(),
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

    pub fn GetLastError () -> u32;

    pub fn FormatMessageA (
        dwFlags: DWORD,             // IN
        lpSource: LPCVOID,          // IN OPT
        dwMessageId: DWORD,         // IN
        dwLanguageId: DWORD,        // IN
        lpBuffer: LPTSTR,           // OUT
        nSize: DWORD,               // IN
        Arguments: *const VA_LIST   // IN OPT
    ) -> DWORD;

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
        name: LPVOID,   // IN
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
        lpOutputBuffer: LPVOID,             // IN
        dwReceveDataLength: u32,            // IN
        dwLocalAddressLength: u32,          // IN
        dwRemoteAddressLength: u32,         // IN
        lpdwBytesReceived: LPDWORD,         // OUT
        lpOverlapped: *mut OVERLAPPED       // IN
    ) -> i32;

    // pub fn WSARecv (
    //     s: SOCKET,                                              // IN
    //     lpBuffers: *mut WSABUF,                                 // IN OUT
    //     dwBufferCount: DWORD,                                   // IN
    //     lpNumberOfBytesRecvd: LPDWORD,                          // OUT
    //     lpFlags: LPDWORD,                                       // IN OUT
    //     lpOverlapped: *mut OVERLAPPED,                          // IN
    //     lpCompletionRoutine: WSAOVERLAPPED_COMPLETION_ROUTINE   // IN
    // ) -> i32;
}


/****************************************************************************
*
*   Endianness
*
***/

#[cfg(target_endian = "little")]
pub mod endian {
    //=======================================================================
    #[inline]
    pub fn net_16 (n: u16) -> u16 {
        ((n & 0xff) << 8) + ((n & 0xff00) >> 8)
    }
}

#[cfg(target_endian = "big")]
pub mod endian {
    //=======================================================================
    #[inline]
    pub fn net_16 (n: u16) -> u16 {
        n
    }
}