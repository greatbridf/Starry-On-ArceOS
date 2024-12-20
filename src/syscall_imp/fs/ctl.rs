use core::ffi::{c_char, c_void};

use arceos_posix_api::ctypes::AT_FDCWD;
use arceos_posix_api::{self as api, ctypes::mode_t};
use axerrno::LinuxError;

use crate::syscall_body;

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
///
/// # Arguments
/// * `fd` - The file descriptor
/// * `op` - The request code. It is of type unsigned long in glibc and BSD,
/// and of type int in musl and other UNIX systems.
/// * `argp` - The argument to the request. It is a pointer to a memory location
pub(crate) fn sys_ioctl(_fd: i32, _op: usize, _argp: *mut c_void) -> i32 {
    syscall_body!(sys_ioctl, {
        warn!("Unimplemented syscall: SYS_IOCTL");
        Ok(0)
    })
}

pub(crate) fn sys_openat(dfd: i32, filename: *const c_char, flags: i32, mode: mode_t) -> i32 {
    match dfd {
        AT_FDCWD => api::sys_open(filename, flags, mode),
        ..0 => -LinuxError::EBADF.code(),
        _ => api::sys_open(filename, flags, mode),
    }
}

pub(crate) fn sys_close(fd: i32) -> i32 {
    api::sys_close(fd)
}

pub(crate) fn sys_dup(fd: i32) -> i32 {
    api::sys_dup(fd)
}

pub(crate) fn sys_dup3(oldfd: i32, newfd: i32, flags: i32) -> i32 {
    // TODO: check `flags` and set F_DUPFD_CLOEXEC if needed
    api::sys_dup2(oldfd, newfd)
}
