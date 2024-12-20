use core::ffi::c_char;

use arceos_posix_api::{self as api, ctypes::mode_t};

pub fn sys_mkdirat(dfd: i32, pathname: *const c_char, mode: mode_t) -> isize {
    api::sys_mkdirat(dfd, pathname, mode) as _
}
