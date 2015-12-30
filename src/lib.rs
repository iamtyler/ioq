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
pub mod net;

mod handle;
mod queue;
mod sys;

pub use self::queue::Custom;
pub use self::queue::Event;
pub use self::queue::Queue;
