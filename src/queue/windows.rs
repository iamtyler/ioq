/**************************************************************************
*
*   queue.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use handle::Handle;
use error::Error;


/****************************************************************************
*
*   Queue
*
***/

pub struct Queue {
    handle: Handle,
}

impl Queue {
    //=======================================================================
    pub fn new () -> Result<Queue, Error> {
        let raw = unsafe {
            os::CreateIoCompletionPort(
                os::INVALID_HANDLE_VALUE,
                os::NULL_HANDLE,
                0,
                0
            )
        };

        if raw.is_null() {
            Err(Error::last_os_error())
        }
        else {
            Ok(Queue {
                handle: Handle::from_raw(raw),
            })
        }
    }
}

impl Drop for Queue {
    //=======================================================================
    fn drop (&mut self) {
        let _ = self.handle.close();
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
#[allow(dead_code)]
pub fn associate (queue: &Queue, handle: Handle) -> Result<(), Error> {
    let success = unsafe {
        os::CreateIoCompletionPort(
            handle.to_raw(),
            queue.handle.to_raw(),
            0,
            0
        )
    } == queue.handle.to_raw();

    if success {
        Ok(())
    }
    else {
        Err(Error::last_os_error())
    }
}


/****************************************************************************
*
*   OS API
*
***/

mod os {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use libc;

    pub type HANDLE = *mut libc::c_void;
    pub type BOOL = i32;
    pub type ULONG_PTR = usize;

    pub const INVALID_HANDLE_VALUE: HANDLE = 0xFFFFFFFFFFFFFFFF as HANDLE;
    pub const NULL_HANDLE: HANDLE = 0 as HANDLE;

    #[repr(C)]
    #[derive(Clone, Debug)]
    pub struct OVERLAPPED {
        pub Internal: ULONG_PTR,
        pub InternalHigh: ULONG_PTR,
        pub Offset: u32,
        pub OffsetHigh: u32,
        pub hEvent: HANDLE,
    }

    #[link(name = "kernel32")]
    extern "stdcall" {
        pub fn CreateIoCompletionPort (
            FileHandle: HANDLE,             // IN
            ExistingCompletionPort: HANDLE, // IN OPT
            CompletionKey: ULONG_PTR,       // IN
            NumberOfConcurrentThreads: u32  // IN
        ) -> HANDLE;
    }
}