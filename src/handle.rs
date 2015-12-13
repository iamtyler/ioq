/**************************************************************************
*
*   handle.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::fmt;
use std::hash;
use std::cmp;

use error::Error;
use sys;


/****************************************************************************
*
*   Handle
*
***/

#[derive(Copy)]
pub struct Handle {
    raw: sys::HANDLE,
}

impl Handle {
    pub fn from_raw (raw: sys::HANDLE) -> Handle { Handle { raw: raw } }
    pub fn to_raw (&self) -> sys::HANDLE { self.raw }
    pub fn into_raw (self) -> sys::HANDLE { self.raw }
    pub fn is_null (&self) -> bool { self.raw.is_null() }
    fn to_usize (&self) -> usize { self.raw as usize }

    //=======================================================================
    pub fn close (self) -> Result<(), Error> {
        if sys::close_handle(self.raw) {
            Ok(())
        }
        else {
            Err(Error::last_os_error())
        }
    }
}

impl Clone for Handle {
    fn clone(&self) -> Handle { *self }
}

impl fmt::Display for Handle {
    //=======================================================================
    fn fmt (&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.to_usize())
    }
}

impl fmt::Debug for Handle {
    //=======================================================================
    fn fmt (&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, fmt)
    }
}

impl PartialEq for Handle {
    //=======================================================================
    fn eq (&self, other: &Handle) -> bool {
        return self.raw == other.raw;
    }
}

impl Eq for Handle {}

impl hash::Hash for Handle {
    //=======================================================================
    fn hash<H: hash::Hasher> (&self, s: &mut H) {
        self.to_usize().hash(s)
    }
}

impl PartialOrd for Handle {
    //=======================================================================
    fn partial_cmp (&self, other: &Handle) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Handle {
    //=======================================================================
    fn cmp (&self, other: &Handle) -> cmp::Ordering {
        self.to_usize().cmp(&other.to_usize())
    }
}
