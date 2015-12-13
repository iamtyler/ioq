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
            win32::CreateIoCompletionPort(
                win32::INVALID_HANDLE_VALUE,
                win32::NULL_HANDLE,
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

    //=======================================================================
    pub fn associate (&self, handle: Handle) -> Result<(), Error> {
        let success = unsafe {
            win32::CreateIoCompletionPort(
                handle.to_raw(),
                self.handle.to_raw(),
                0,
                0
            )
        } == self.handle.to_raw();

        if success {
            Ok(())
        }
        else {
            Err(Error::last_os_error())
        }
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
