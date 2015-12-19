/****************************************************************************
*
*   net/init.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use sys;
use error::Error;


/****************************************************************************
*
*   InitGuard
*
***/

pub struct InitGuard;

impl Drop for InitGuard {
    fn drop (&mut self) {
        unsafe { sys::WSACleanup() };
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn initialize () -> Result<InitGuard, Error> {
    let mut data = sys::WSAData::new();
    let code = unsafe {
        sys::WSAStartup(
            2 + (2 << 8),
            &mut data as *mut sys::WSAData
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