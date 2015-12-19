/****************************************************************************
*
*   net/init.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use error::Error;


/****************************************************************************
*
*   InitGuard
*
***/

pub struct InitGuard;

impl Drop for InitGuard {
    fn drop (&mut self) {
        unsafe { os::WSACleanup() };
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn initialize () -> Result<InitGuard, Error> {
    let mut data = os::WSAData::new();
    let code = unsafe {
        os::WSAStartup(
            2 + (2 << 8),
            &mut data as *mut os::WSAData
        )
    };

    if code != 0 {
        return Err(Error::from_os_error_code(code));
    }

    // TODO: verify version

    Ok(InitGuard)
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

    const WSADESCRIPTION_LEN: usize = 256;
    const WSASYS_STATUS_LEN: usize = 128;

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

    #[link(name = "Ws2_32")]
    extern "stdcall" {
        pub fn WSAStartup (
            wVersionRequested: u16, // IN
            lpWSAData: *mut WSAData // OUT
        ) -> i32;

        pub fn WSACleanup () -> i32;
    }
}


/****************************************************************************
*
*   Tests
*
***/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_guard () {
        let guard = initialize();
        assert!(guard.is_ok());
    }
}