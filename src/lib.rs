/**************************************************************************
*
*   lib.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate libc;

pub mod error;
//pub mod net;

mod handle;
mod queue;
//mod win32;

pub use self::queue::Queue;
