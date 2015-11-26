/**************************************************************************
*
*   queue.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use win32;


/****************************************************************************
*
*   Types
*
***/

#[cfg(target_pointer_width = "32")]
pub type Handle = u32;
#[cfg(target_pointer_width = "64")]
pub type Handle = u64;


/****************************************************************************
*
*   Linkable
*
***/

pub trait Linkable {
    fn handle (&self) ->  {}
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
    pub fn new () -> Option<Queue> {
        let handle = unsafe {
            win32::CreateIoCompletionPort(
                win32::INVALID_HANDLE_VALUE,
                win32::NULL_HANDLE,
                0,
                0
            )
        };
        if handle.is_null() {
            return None;
        }

        return Some(Queue {
            handle: handle as Handle
        });
    }

    pub fn link (&mut self, handle: Handle) -> bool {
        return unsafe {
            win32::CreateIoCompletionPort(
                handle as win32::HANDLE,
                self.handle as win32::HANDLE,
                0, // TODO: pass in link notify
                0
            ) as Handle
        } == self.handle;
    }
}
