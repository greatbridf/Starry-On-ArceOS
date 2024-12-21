use arceos_posix_api::{self as api, ctypes::pid_t};
use axerrno::LinuxError;

use axtask::{current, TaskExtRef};
use bitflags::bitflags;
use memory_addr::VirtAddr;

use crate::task::{clone_user_task, spawn_user_task};

pub(crate) fn sys_sched_yield() -> i32 {
    api::sys_sched_yield()
}

pub(crate) fn sys_nanosleep(
    req: *const api::ctypes::timespec,
    rem: *mut api::ctypes::timespec,
) -> i32 {
    unsafe { api::sys_nanosleep(req, rem) }
}

pub(crate) fn sys_wait4(pid: pid_t, status: *mut i32, options: i32, rusage: *mut ()) -> pid_t {
    -22
}

bitflags! {
    #[derive(Debug)]
    struct CloneFlags: i32 {
        const CLONE_VM = 0x00000100;
        const CLONE_FS = 0x00000200;
        const CLONE_FILES = 0x00000400;
        const CLONE_SIGHAND = 0x00000800;
        const CLONE_PTRACE = 0x00002000;
        const CLONE_VFORK = 0x00004000;
        const CLONE_PARENT = 0x00008000;
        const CLONE_THREAD = 0x00010000;
        const CLONE_NEWNS = 0x00020000;
        const CLONE_SYSVSEM = 0x00040000;
        const CLONE_SETTLS = 0x00080000;
        const CLONE_PARENT_SETTID = 0x00100000;
        const CLONE_CHILD_SETTID = 0x00200000;
        const CLONE_DETACHED = 0x00400000;
        const CLONE_UNTRACED = 0x00800000;
        const CLONE_CHILD_CLEARTID = 0x02000000;
        const CLONE_NEWUTS = 0x04000000;
        const CLONE_NEWIPC = 0x08000000;
        const CLONE_NEWUSER = 0x10000000;
        const CLONE_NEWPID = 0x20000000;
        const CLONE_NEWNET = 0x40000000;
        const CLONE_IO = 0x80000000u32 as i32;
    }
}

fn do_sys_clone(
    flags: CloneFlags,
    child_stack: VirtAddr,
    _ptid: *mut pid_t,
    _ctid: *mut pid_t,
    _tls: VirtAddr,
) -> pid_t {
    // TODO: We ignore all the flags for now, only do what `fork` will do.
    let current = current();

    let new_task = match clone_user_task(current.as_task_ref(), child_stack) {
        Ok(task) => task,
        Err(e) => {
            return -LinuxError::from(e).code() as _;
        }
    };

    let status = new_task.join();
    info!(
        "clone: new task({}) exited with status: {:?}",
        new_task.task_ext().proc_id,
        status
    );

    new_task.task_ext().proc_id as _
}

#[cfg(target_arch = "x86_64")]
pub(crate) fn sys_clone(
    flags: i32,
    child_stack: VirtAddr,
    _ptid: *mut pid_t,
    _ctid: *mut pid_t,
    _newtls: VirtAddr,
) -> pid_t {
    let flags = CloneFlags::from_bits_truncate(flags);
    if child_stack.as_usize() == 0 {
        return -LinuxError::EINVAL.code() as _;
    }
    do_sys_clone(flags, child_stack, _ptid, _ctid, _newtls)
}

#[cfg(not(target_arch = "x86_64"))]
pub(crate) fn sys_clone(
    flags: i32,
    child_stack: VirtAddr,
    _ptid: *mut pid_t,
    _newtls: VirtAddr,
    _ctid: *mut pid_t,
) -> pid_t {
    let flags = CloneFlags::from_bits_truncate(flags);
    if child_stack.as_usize() == 0 {
        return -LinuxError::EINVAL.code() as _;
    }
    do_sys_clone(flags, child_stack, _ptid, _ctid, _newtls)
}
