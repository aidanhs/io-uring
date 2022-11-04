use core::{ffi, ptr};
use core::sync::atomic;

use rustix::io;

/// A region of memory mapped using `mmap(2)`.
pub struct Mmap {
    addr: ptr::NonNull<ffi::c_void>,
    len: usize,
}

impl Mmap {
    /// Map `len` bytes starting from the offset `offset` in the file descriptor `fd` into memory.
    pub fn new(fd: &OwnedFd, offset: u64, len: usize) -> io::Result<Mmap> {
        unsafe {
            match rustix::mm::mmap(
                ptr::null_mut(),
                len,
                rustix::mm::ProtFlags::READ | rustix::mm::ProtFlags::WRITE,
                rustix::mm::MapFlags::SHARED | rustix::mm::MapFlags::POPULATE,
                fd,
                offset,
            ) {
                Ok(addr) => {
                    // here, `mmap` will never return null
                    let addr = ptr::NonNull::new_unchecked(addr);
                    Ok(Mmap { addr, len })
                }
                Err(e) => Err(e),
            }
        }
    }

    /// Do not make the stored memory accessible by child processes after a `fork`.
    pub fn dontfork(&self) -> io::Result<()> {
        unsafe { rustix::mm::madvise(self.addr.as_ptr(), self.len, rustix::mm::Advice::LinuxDontFork) }
    }

    /// Get a pointer to the memory.
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut ffi::c_void {
        self.addr.as_ptr()
    }

    /// Get a pointer to the data at the given offset.
    #[inline]
    pub unsafe fn offset(&self, offset: u32) -> *mut ffi::c_void {
        self.as_mut_ptr().add(offset as usize)
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe {
            let _ = rustix::mm::munmap(self.addr.as_ptr(), self.len);
        }
    }
}

pub use fd::OwnedFd;

#[cfg(feature = "io_safety")]
mod fd {
    pub use std::os::unix::io::OwnedFd;
}

#[cfg(not(feature = "io_safety"))]
mod fd {
    pub use rustix::fd::OwnedFd;
    //use core::mem;
    //use rustix::fd::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

    ///// API-compatible with the `OwnedFd` type in the Rust stdlib.
    //pub struct OwnedFd(RawFd);

    //impl AsRawFd for OwnedFd {
    //    #[inline]
    //    fn as_raw_fd(&self) -> RawFd {
    //        self.0
    //    }
    //}

    //impl IntoRawFd for OwnedFd {
    //    #[inline]
    //    fn into_raw_fd(self) -> RawFd {
    //        let fd = self.0;
    //        mem::forget(self);
    //        fd
    //    }
    //}

    //impl FromRawFd for OwnedFd {
    //    #[inline]
    //    unsafe fn from_raw_fd(fd: RawFd) -> OwnedFd {
    //        OwnedFd(fd)
    //    }
    //}

    //impl Drop for OwnedFd {
    //    fn drop(&mut self) {
    //        unsafe {
    //            libc::close(self.0);
    //        }
    //    }
    //}
}

#[inline(always)]
pub unsafe fn unsync_load(u: *const atomic::AtomicU32) -> u32 {
    *u.cast::<u32>()
}

#[inline]
pub const fn cast_ptr<T>(n: &T) -> *const T {
    n
}
