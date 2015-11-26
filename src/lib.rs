/**************************************************************************
*
*   lib.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate libc;

mod win32;
mod queue;

pub use queue::Queue;
