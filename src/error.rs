/**************************************************************************
*
*   queue.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::error;
use std::fmt;
use sys;


/****************************************************************************
*
*   Error
*
***/

#[derive(Debug)]
pub struct Error {
    inner: Inner
}

impl Error {
    //=======================================================================
    pub fn new<E> (kind: ErrorKind, error: E) -> Error
        where E: Into<Box<error::Error + Send + Sync>>
    {
        Error {
            inner: Inner::Custom(Box::new(Custom {
                kind: kind,
                error: error.into(),
            }))
        }
    }

    //=======================================================================
    pub fn last_os_error () -> Error {
        Error::from_os_error_code(sys::last_error_code())
    }

    //=======================================================================
    pub fn from_os_error_code (code: i32) -> Error {
        Error { inner: Inner::Os(code) }
    }

    //=======================================================================
    pub fn os_error_code (&self) -> Option<i32> {
        match self.inner {
            Inner::Os(c) => Some(c),
            Inner::Custom(..) => None,
        }
    }

    //=======================================================================
    pub fn kind (&self) -> Option<ErrorKind> {
        match self.inner {
            Inner::Os(..) => None,
            Inner::Custom(ref c) => Some(c.kind),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            Inner::Os(code) => {
                let detail = ""; // sys::os::error_string(code);
                write!(fmt, "{} (os error {})", detail, code)
            }
            Inner::Custom(ref c) => c.error.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description (&self) -> &str {
        match self.inner {
            Inner::Os(..) => "os error",
            Inner::Custom(ref c) => c.error.description(),
        }
    }
}


/****************************************************************************
*
*   Inner
*
***/

enum Inner {
    Os(i32),
    Custom(Box<Custom>),
}

impl fmt::Debug for Inner {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Inner::Os(ref code) =>
                fmt.debug_struct("Os").field("code", code).finish(),
                   //.field("message", &sys::os::error_string(*code)).finish(),
            Inner::Custom(ref c) => fmt.debug_tuple("Custom").field(c).finish(),
        }
    }
}


/****************************************************************************
*
*   Custom
*
***/

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<error::Error + Send + Sync>,
}


/****************************************************************************
*
*   ErrorKind
*
***/

#[derive(Copy, PartialEq, Eq, Clone, Debug)]
pub enum ErrorKind {
    Unknown,
}
