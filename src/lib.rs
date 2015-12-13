/**************************************************************************
*
*   lib.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

extern crate libc;

pub mod handle;
pub mod error;
//pub mod net;

mod queue;
mod sys;
//mod win32;
//mod event;

pub use self::queue::Queue;
