/****************************************************************************
*
*   net/mod.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

mod endpoint;
mod tcp;

use super::win32;


/****************************************************************************
*
*   Expose stuff
*
***/

pub use self::endpoint::IpAddrV4;
pub use self::endpoint::EndpointV4;
pub use self::endpoint::Endpoint;

pub use self::tcp::TcpListener;
pub use self::tcp::TcpStream;


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn initialize () -> bool {
    let mut data = win32::WSAData {
        wVersion: 0,
        wHighVersion: 0,
        szDescription: [0; win32::WSADESCRIPTION_LEN + 1],
        szSystemStatus: [0; win32::WSASYS_STATUS_LEN + 1],
        iMaxSockets: 0,
        iMaxUdpDg: 0,
        lpVendorInfo: 0 as *mut u8,
    };

    if unsafe { win32::WSAStartup(
        2 + (2 << 8),
        &mut data as *mut win32::WSAData
    ) } != 0 {
        return false;
    }

    // TODO: verify version

    return true;
}

//===========================================================================
pub fn cleanup () {
    // TODO: return detailed error
    unsafe {
        win32::WSACleanup();
    }
}
