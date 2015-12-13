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
*   OS API
*
***/

mod os {
    #[link(name = "kernel32")]
    extern "stdcall" {
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
