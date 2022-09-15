#![allow(unused_macros, dead_code)]

use std::ffi::{CStr, CString, NulError};
#[cfg(target_os = "android")]
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

#[cfg(target_os = "android")]
#[link(name = "ashmem", kind = "static")]
extern "C" {
    fn ashmem_open(name: *const libc::c_char, size: libc::size_t) -> libc::c_int;
    fn ashmem_get_name(
        fd: libc::c_int,
        buf: *mut libc::c_char,
        size: libc::size_t,
    ) -> *const libc::c_char;
    fn ashmem_get_size(fd: libc::c_int) -> libc::size_t;
    fn test();
}

#[cfg(target_os = "android")]
pub fn test_in_rust() {
    unsafe { test(); }
}

pub struct Shm {
    #[cfg(target_os = "android")]
    fd: libc::c_int,
    addr: *mut libc::c_void,
    size: usize,
    name: String,
}

impl Shm {
    #[cfg(target_os = "android")]
    pub fn map(fd: libc::c_int) -> Result<Shm> {
        unsafe { Self::init(fd, Self::get_name(fd)?, Self::get_size(fd)) }
    }

    pub fn new(name: &str, size: usize) -> Result<Shm> {
        unsafe {
            #[cfg(target_os = "android")]
            let fd = ashmem_open(CString::new(name)?.as_ptr(), size as _);

            #[cfg(not(target_os = "android"))]
            let fd = libc::shm_open(
                CString::new(name)?.as_ptr(),
                libc::O_RDWR | libc::O_CREAT,
                0o666,
            );

            if fd == -1 {
                #[cfg(target_os = "android")]
                return_errno!("ashmem_open");

                #[cfg(not(target_os = "android"))]
                return_errno!("shm_open");
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> usize {
        self.size
    }

    #[cfg(target_os = "android")]
    pub fn fd(&self) -> libc::c_int {
        self.fd
    }

    pub fn as_ptr(&self) -> *const libc::c_void {
        self.addr
    }

    pub fn as_mut_ptr(&mut self) -> *mut libc::c_void {
        self.addr
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

        #[cfg(not(target_os = "android"))]
        if libc::close(fd) == -1 {
            return_errno!("close");
        }

        Ok(Shm {
            #[cfg(target_os = "android")]
            fd,
            addr,
            size,
            name,
        })
    }

    #[cfg(target_os = "android")]
    fn get_name(fd: libc::c_int) -> Result<String> {
        unsafe {
            let mut buf: [libc::c_char; 256] = MaybeUninit::uninit().assume_init();
            let buf = ashmem_get_name(fd, buf.as_mut_ptr(), 256);
            let c_str = CStr::from_ptr(buf);
            Ok(c_str.to_str()?.to_string())
        }
    }

    #[cfg(target_os = "android")]
    fn get_size(fd: libc::c_int) -> usize {
        unsafe { ashmem_get_size(fd) as _ }
    }
}

impl Drop for Shm {
    fn drop(&mut self) {
        unsafe {
            assert_ne!(libc::munmap(self.addr, self.size as _), -1);

            #[cfg(target_os = "android")]
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
