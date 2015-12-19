/****************************************************************************
*
*   sys/mod.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;