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


/****************************************************************************
*
*   Types
*
***/

pub use self::os::HANDLE;


/****************************************************************************
*
*   OS API
*
***/

mod os {
    use libc;

    pub type HANDLE = *mut libc::c_void;
    pub type BOOL = i32;

    #[link(name = "kernel32")]
    extern "stdcall" {
        pub fn CloseHandle (
            hObject: HANDLE // IN
        ) -> BOOL;

        pub fn GetLastError () -> u32;
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn last_error_code () -> i32 {
    (unsafe { os::GetLastError() } as i32)
}

//===========================================================================
pub fn close_handle (handle: HANDLE) -> bool {
    (unsafe { os::CloseHandle(handle) } != 0)
}
