#![allow(unused_macros, dead_code)]

use std::ffi::{CStr, CString, NulError};
use std::ptr;
use std::str::Utf8Error;

macro_rules! panic_errno {
    ($msg: expr) => {{
        let errno = $crate::libc_errno();
        panic!("{}: {}({})", $msg, $crate::strerror(errno).unwrap(), errno)
    }};
}

macro_rules! return_errno {
    ($msg: expr) => {{
        let errno = $crate::libc_errno();
        return Err($crate::Error::Errno(
            errno,
            format!("{}: {}", $msg, $crate::strerror(errno)?),
        ));
    }};

    () => {
        let errno = $crate::libc_errno();
        return Err($crate::Error::Errno(errno, $crate::strerror(errno)?));
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("errno: {0}, msg: {1}")]
    Errno(libc::c_int, String),

    #[error("UTF8 string error: {0}")]
    Utf8(#[from] Utf8Error),

    #[error("CString error: {0}")]
    Nul(#[from] NulError),
}

#[link(name = "ashmem", kind = "static")]
extern "C" {
    fn ashmem_open(name: *const libc::c_char, size: libc::size_t) -> libc::c_int;
}

pub struct Shm {
    fd: libc::c_int,
    addr: *mut libc::c_void,
    size: usize,
    name: String,
}

impl Shm {
    pub fn new(name: &str, size: usize) -> Result<Shm> {
        unsafe {
            let fd = ashmem_open(CString::new(name)?.as_ptr(), size as _);
            if fd == -1 {
                return_errno!("ashmem_open");
            }
            let addr = libc::mmap(
                ptr::null_mut::<libc::c_void>(),
                size as _,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            if addr == libc::MAP_FAILED {
                libc::close(fd);
                return_errno!("mmap");
            }

            Ok(Shm {
                fd,
                addr,
                size,
                name: name.to_string(),
            })
        }
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        unsafe {
            assert_ne!(libc::munmap(self.addr, self.size as _), -1);
            assert_ne!(libc::close(self.fd), -1);
        }
    }
}

fn libc_errno() -> libc::c_int {
    unsafe {
        #[cfg(target_os = "android")]
        return *libc::__errno();

        #[cfg(not(target_os = "android"))]
        *libc::__errno_location()
    }
}

fn strerror(errno: i32) -> Result<String> {
    unsafe {
        let cstr = CStr::from_ptr(libc::strerror(errno as _));
        Ok(cstr.to_str()?.to_string())
    }
}
