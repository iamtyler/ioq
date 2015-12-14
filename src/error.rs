/**************************************************************************
*
*   error.rs
*   ioq
*
*   Copyright 2015 Tyler Cole
*
***/

use std::error;
use std::fmt;


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
                let detail = sys::error_string(code);
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
                   .field("message", &sys::error_string(*code)).finish(),
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


/****************************************************************************
*
*   OS API
*
***/

#[cfg(windows)]
mod sys {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use libc;

    use std::cell::RefCell;
    use std::mem;
    use std::ptr;

    type DWORD = u32;
    type LPCVOID = *const libc::c_void;
    type LPTSTR = *mut u8;
    type va_list = *mut libc::c_char;

    const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x00001000;
    const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x00000200;
    const FORMAT_MESSAGE_MAX_WIDTH_MASK: u32 = 0x000000FF;

    #[link(name = "kernel32")]
    extern "stdcall" {
        fn GetLastError () -> u32;

        fn FormatMessageA (
            dwFlags: DWORD,             // IN
            lpSource: LPCVOID,          // IN OPT
            dwMessageId: DWORD,         // IN
            dwLanguageId: DWORD,        // IN
            lpBuffer: LPTSTR,           // OUT
            nSize: DWORD,               // IN
            Arguments: *const va_list   // IN OPT
        ) -> DWORD;
    }

    thread_local!(static MESSAGE: RefCell<[u8; 64]> = RefCell::new([0; 64]));

    //=======================================================================
    pub fn last_error_code () -> i32 {
        (unsafe { GetLastError() } as i32)
    }

    //=======================================================================
    pub fn error_string<'a> (code: i32) -> &'a str {
        let mut message: &str = "";

        MESSAGE.with(|m| {
            let mut buffer: &mut [u8] = &mut*m.borrow_mut();

            let count = unsafe {
                FormatMessageA(
                      FORMAT_MESSAGE_FROM_SYSTEM
                    | FORMAT_MESSAGE_IGNORE_INSERTS
                    | FORMAT_MESSAGE_MAX_WIDTH_MASK,
                    ptr::null(),
                    code as DWORD,
                    0,
                    buffer.as_mut_ptr(),
                    buffer.len() as DWORD,
                    ptr::null()
                )
            };

            message = unsafe { mem::transmute::<&[u8], &str>(&buffer[..count as usize]) };
        });

        return message;
    }
}