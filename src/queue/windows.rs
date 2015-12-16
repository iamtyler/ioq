/**************************************************************************
*
*   queue.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::mem;
use std::ptr;

use handle::Handle;
use error::Error;
use super::{Context, Custom, Event};


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
                ptr::null_mut(),
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

    //=======================================================================
    pub fn enqueue (&self, custom: Box<Custom>) -> Result<(), Error> {
        // Create state
        let context = Box::new(CustomContext::new(custom));
        let state = Box::new(State::new(context));
        let overlapped = state.overlapped_raw();

        // 
        let success = unsafe {
            os::PostQueuedCompletionStatus(
                self.handle.to_raw(),
                0,
                ptr::null_mut(),
                overlapped
            ) != 0
        };

        // Handle error
        if !success {
            return Err(Error::last_os_error());
        }

        // Take ownership of memory
        Box::into_raw(state);
        return Ok(());
    }

    //=======================================================================
    pub fn dequeue (&self) -> Result<Event, Error> {
        // Output data
        let mut bytes: u32 = 0;
        let mut key: os::ULONG_PTR = ptr::null_mut();
        let mut overlapped: *mut os::OVERLAPPED = ptr::null_mut();

        // Get completion data
        let success = unsafe {
            os::GetQueuedCompletionStatus(
                self.handle.to_raw(),
                &mut bytes as *mut u32,
                &mut key as *mut os::ULONG_PTR,
                &mut overlapped as *mut *mut os::OVERLAPPED,
                os::INFINITE
            ) != 0
        };

        // Handle error
        if !success {
            return Err(Error::last_os_error());
        }

        // Get state and return event
        let state = unsafe { State::from_overlapped_raw(overlapped) };
        Ok(state.into_event(bytes))
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
*   CustomContext
*
***/

struct CustomContext {
    custom: Box<Custom>,
}

impl CustomContext {
    //=======================================================================
    pub fn new (custom: Box<Custom>) -> CustomContext {
        CustomContext {
            custom: custom
        }
    }
}

impl Context for CustomContext {
    //=======================================================================
    fn to_event (self: Box<Self>, bytes: u32) -> Event {
        let _ = bytes;
        Event::Custom(self.custom)
    }
}


/****************************************************************************
*
*   State
*
***/

#[repr(C)]
pub struct State {
    overlapped: os::OVERLAPPED,
    context: Box<Context>,
}

impl State {
    //=======================================================================
    pub fn new (context: Box<Context>) -> State {
        State {
            overlapped: os::OVERLAPPED::new(),
            context: context,
        }
    }

    //=======================================================================
    fn into_event (self, bytes: u32) -> Event {
        self.context.to_event(bytes)
    }

    //=======================================================================
    unsafe fn from_overlapped_raw (overlapped: *mut os::OVERLAPPED) -> Box<State> {
        let state: *mut State = mem::transmute(overlapped);
        Box::from_raw(state)
    }

    //=======================================================================
    fn overlapped_raw (&self) -> *mut os::OVERLAPPED {
        &self.overlapped as *const _ as *mut _
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
            ptr::null_mut(),
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

    use std::ptr;

    use libc;

    pub type HANDLE = *mut libc::c_void;
    pub type BOOL = i32;
    pub type ULONG_PTR = *mut u32;

    pub const INVALID_HANDLE_VALUE: HANDLE = 0xFFFFFFFFFFFFFFFF as HANDLE;
    pub const NULL_HANDLE: HANDLE = 0 as HANDLE;

    pub const INFINITE: u32 = 0xFFFFFFFF;

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

    #[link(name = "kernel32")]
    extern "stdcall" {
        pub fn CreateIoCompletionPort (
            FileHandle: HANDLE,             // IN
            ExistingCompletionPort: HANDLE, // IN OPT
            CompletionKey: ULONG_PTR,       // IN
            NumberOfConcurrentThreads: u32  // IN
        ) -> HANDLE;

        pub fn PostQueuedCompletionStatus (
            CompletionPort: HANDLE,             // IN
            dwNumberOfBytesTransferred: u32,    // IN
            dwCompletionKey: ULONG_PTR,         // IN
            lpOverlapped: *mut OVERLAPPED       // IN OPT
        ) -> BOOL;

        pub fn GetQueuedCompletionStatus (
            CompletionPort: HANDLE,             // IN
            lpNumberOfBytes: *mut u32,          // OUT
            lpCompletionKey: *mut ULONG_PTR,    // OUT
            lpOverlapped: *mut *mut OVERLAPPED, // OUT
            dwMilliseconds: u32                 // IN
        ) -> BOOL;
    }
}