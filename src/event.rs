/**************************************************************************
*
*   event.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/


/****************************************************************************
*
*   Types
*
***/

type EventGenerator = fn (bytes: i32) -> Event;


/****************************************************************************
*
*   EventContext
*
***/

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EventContext {
    overlapped: win32::OVERLAPPED,
}

impl EventContext {
    //=======================================================================
    pub fn new () -> EventContext {
        EventContext {
            overlapped: win32::OVERLAPPED::new(),
        }
    }

    //=======================================================================
    pub fn overlapped (&mut self) -> &mut win32::OVERLAPPED {
        &mut self.overlapped
    }

    //=======================================================================
    pub fn overlapped_ptr (&self) -> *mut win32::OVERLAPPED {
        &self.overlapped as *const _ as *mut _
    }
}


/****************************************************************************
*
*   Event
*
***/

pub enum Event {
    Custom,
}