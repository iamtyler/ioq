/****************************************************************************
*
*   queue/mod.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;


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
        assert!(Queue::new().is_ok());
    }
}