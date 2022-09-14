#![allow(unused_macros, dead_code)]

use std::ffi::{CStr, CString, NulError};
use std::mem::MaybeUninit;
use std::str::Utf8Error;
use std::{ptr, slice};

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
    Null(#[from] NulError),
}

#[link(name = "ashmem", kind = "static")]
extern "C" {
    fn ashmem_open(name: *const libc::c_char, size: libc::size_t) -> libc::c_int;
    fn ashmem_get_name(
        fd: libc::c_int,
        buf: *mut libc::c_char,
        size: libc::size_t,
    ) -> *const libc::c_char;
    fn ashmem_get_size(fd: libc::c_int) -> libc::size_t;
}

pub struct Shm {
    fd: libc::c_int,
    addr: *mut libc::c_void,
    size: usize,
    name: String,
}

impl Shm {
    pub fn map(fd: libc::c_int) -> Result<Shm> {
        unsafe { Self::init(fd, Self::get_name(fd)?, Self::get_size(fd)) }
    }

    pub fn new(name: &str, size: usize) -> Result<Shm> {
        unsafe {
            let fd = ashmem_open(CString::new(name)?.as_ptr(), size as _);
            if fd == -1 {
                return_errno!("ashmem_open");
            }
            Self::init(fd, name.to_string(), size)
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.addr as *const u8, self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.addr as *mut u8, self.size) }
    }

    unsafe fn init(fd: libc::c_int, name: String, size: usize) -> Result<Shm> {
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
            name,
        })
    }

    fn get_name(fd: libc::c_int) -> Result<String> {
        unsafe {
            let mut buf: [libc::c_char; 256] = MaybeUninit::uninit().assume_init();
            let buf = ashmem_get_name(fd, buf.as_mut_ptr(), 256);
            let c_str = CStr::from_ptr(buf);
            Ok(c_str.to_str()?.to_string())
        }
    }

    fn get_size(fd: libc::c_int) -> usize {
        unsafe { ashmem_get_size(fd) as _ }
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
