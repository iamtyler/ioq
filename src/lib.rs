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

mod win32;
mod queue;
mod sys;
//mod event;

pub use self::queue::Queue;
