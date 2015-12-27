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

use net;
use sys;
use handle::Handle;
use error::Error;


/****************************************************************************
*
*   Traits
*
***/

pub trait Context {
    fn into_event (self: Box<Self>, bytes: u32) -> Event;
    fn into_error (self: Box<Self>, bytes: u32) -> Event;
}

pub trait Custom {
    fn execute (self: Box<Self>);
}


/****************************************************************************
*
*   Event
*
***/

#[derive(Debug)]
pub enum Event {
    Custom,
    TcpAccept(Result<net::TcpStream, Error>),
}


/****************************************************************************
*
*   State
*
***/

#[repr(C)]
pub struct State {
    overlapped: sys::OVERLAPPED,
    context: Box<Context>,
}

impl State {
    //=======================================================================
    pub fn new (context: Box<Context>) -> State {
        let raw = Box::into_raw(context);
        
        State {
            overlapped: sys::OVERLAPPED::new(),
            context: unsafe { Box::from_raw(raw) },
        }
    }

    //=======================================================================
    fn into_context (self) -> Box<Context> {
        let raw = Box::into_raw(self.context);
        unsafe { Box::from_raw(raw) }
    }

    //=======================================================================
    unsafe fn from_overlapped_raw (overlapped: *mut sys::OVERLAPPED) -> Box<State> {
        let state: *mut State = mem::transmute(overlapped);
        Box::from_raw(state)
    }

    //=======================================================================
    pub fn overlapped_raw (&self) -> *mut sys::OVERLAPPED {
        let raw = &self.overlapped as *const _ as *mut _;
        raw
    }
}


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
            sys::CreateIoCompletionPort(
                sys::INVALID_HANDLE_VALUE,
                sys::NULL_HANDLE,
                ptr::null_mut(),
                0
            )
        };

        if raw.is_null() {
            Err(Error::os_error())
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

        // Post event
        let success = unsafe {
            sys::PostQueuedCompletionStatus(
                self.handle.to_raw(),
                0,
                ptr::null_mut(),
                overlapped
            ) != 0
        };

        // Handle error
        if !success {
            return Err(Error::os_error());
        }

        // Take ownership of memory
        Box::into_raw(state);
        return Ok(());
    }

    //=======================================================================
    pub fn dequeue (&self) -> Result<Event, Error> {
        // Output data
        let mut bytes: u32 = 0;
        let mut key: sys::ULONG_PTR = ptr::null_mut();
        let mut overlapped: *mut sys::OVERLAPPED = ptr::null_mut();

        // Get completion data
        let success = unsafe {
            sys::GetQueuedCompletionStatus(
                self.handle.to_raw(),
                &mut bytes as *mut u32,
                &mut key as *mut sys::ULONG_PTR,
                &mut overlapped as *mut *mut sys::OVERLAPPED,
                sys::INFINITE
            ) != 0
        };

        if success || overlapped != ptr::null_mut() {
            let state = unsafe { State::from_overlapped_raw(overlapped) };
            let context = state.into_context();

            if success {
                Ok(context.into_event(bytes))
            }
            else {
                Ok(context.into_error(bytes))
            }
        }
        else {
            Err(Error::os_error())
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
    fn into_event (self: Box<Self>, _: u32) -> Event {
        self.custom.execute();
        Event::Custom
    }

    //=======================================================================
    fn into_error (self: Box<Self>, _: u32) -> Event {
        self.custom.execute();
        Event::Custom
    }
}


/****************************************************************************
*
*   Public functions
*
***/

//===========================================================================
pub fn associate (queue: &Queue, handle: Handle) -> Result<(), Error> {
    let success = unsafe {
        sys::CreateIoCompletionPort(
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
        Err(Error::os_error())
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

    const NUMBER: u32 = 1234;

    struct TestEvent {
        n: u32,
    }

    impl Custom for TestEvent {
        fn execute (self: Box<Self>) {
            assert_eq!(NUMBER, self.n);
        }
    }

    //=======================================================================
    #[test]
    fn create_queue () {
        assert!(Queue::new().is_ok());
    }

    //=======================================================================
    #[test]
    fn custom_event () {
        let queue = Queue::new();
        assert!(queue.is_ok());
        let queue = queue.unwrap();

        let event = Box::new(TestEvent { n: NUMBER });
        assert!(queue.enqueue(event).is_ok());

        let event = queue.dequeue();
        assert!(event.is_ok());
        match event.unwrap() {
            Event::Custom => {},
            _ => panic!("Expected Event::Custom"),
        }
    }
}