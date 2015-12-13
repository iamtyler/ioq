/**************************************************************************
*
*   queue.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use win32;
use handle::Handle;


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
    pub fn new () -> Option<Queue> {
        let raw = unsafe {
            win32::CreateIoCompletionPort(
                win32::INVALID_HANDLE_VALUE,
                win32::NULL_HANDLE,
                0,
                0
            )
        };
        if raw.is_null() {
            return None;
        }

        return Some(Queue {
            handle: Handle::from_raw(raw),
        });
    }

    //=======================================================================
    pub fn link (&self, handle: Handle) -> bool {
        return unsafe {
            win32::CreateIoCompletionPort(
                handle.to_raw(),
                self.handle.to_raw(),
                0,
                0
            )
        } == self.handle.to_raw();
    }
}

impl Drop for Queue {
    //=======================================================================
    fn drop (&mut self) {
        self.handle.close();
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

    //=======================================================================
    #[test]
    fn create_queue () {
        Queue::new().unwrap();
    }
}
