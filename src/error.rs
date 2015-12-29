/**************************************************************************
*
*   error.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::cell::RefCell;
use std::error;
use std::fmt;
use std::mem;
use std::ptr;

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
    pub fn unknown () -> Error {
        Error::new(ErrorKind::Unknown, "Unknown error")
    }

    //=======================================================================
    pub fn not_implemented () -> Error {
        Error::new(ErrorKind::NotImplemented, "Functionality not implemented")
    }

    //=======================================================================
    pub fn os_error_code (&self) -> Option<i32> {
        match self.inner {
            Inner::Os(c) => Some(c),
            Inner::Custom(..) => None,
        }
    }

    //=======================================================================
    pub fn from_os_error_code (code: i32) -> Error {
        Error { inner: Inner::Os(code) }
    }

    //=======================================================================
    pub fn os_error () -> Error {
        Error::from_os_error_code(last_error_code())
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
                let detail = error_string(code);
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
                fmt.debug_struct("Os").field("code", code)
                   .field("message", &error_string(*code)).finish(),
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
    NotImplemented,
}


/****************************************************************************
*
*   Public functions
*
***/

const MESSAGE_BYTES: usize = 128;
thread_local!(static MESSAGE: RefCell<[u8; MESSAGE_BYTES]> = RefCell::new([0; MESSAGE_BYTES]));

//=======================================================================
pub fn last_error_code () -> i32 {
    (unsafe { sys::GetLastError() } as i32)
}

//=======================================================================
pub fn error_string<'a> (code: i32) -> &'a str {
    let mut message: &str = "";

    MESSAGE.with(|m| {
        let mut buffer: &mut [u8] = &mut *m.borrow_mut();

        let count = unsafe {
            sys::FormatMessageA(
                  sys::FORMAT_MESSAGE_FROM_SYSTEM
                | sys::FORMAT_MESSAGE_IGNORE_INSERTS
                | sys::FORMAT_MESSAGE_MAX_WIDTH_MASK,
                ptr::null(),
                code as sys::DWORD,
                0,
                buffer.as_mut_ptr(),
                buffer.len() as sys::DWORD,
                ptr::null()
            )
        };

        message = unsafe { mem::transmute(&buffer[..count as usize]) };
    });

    if message.len() == 0 && last_error_code() == sys::ERROR_INSUFFICIENT_BUFFER {
        message = "[MESSAGE buffer not large enough]";
    }

    return message;
}