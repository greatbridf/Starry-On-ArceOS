#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate log;
extern crate alloc;
extern crate axstd;

#[rustfmt::skip]
mod config {
    include!(concat!(env!("OUT_DIR"), "/uspace_config.rs"));
}
mod loader;
mod mm;
mod syscall_imp;
mod task;

use alloc::vec;
use alloc::{string::String, sync::Arc};

use axhal::arch::UspaceContext;
use axsync::Mutex;

#[no_mangle]
fn main() {
    let names = [
        "brk",
        "chdir",
        "clone",
        "close",
        "dup2",
        "dup",
        "execve",
        "exit",
        "fork",
        "fstat",
        "getcwd",
        "getdents",
        "getpid",
        "getppid",
        "gettimeofday",
        "mkdir_",
        "mmap",
        "mount",
        "munmap",
        "openat",
        "open",
        "pipe",
        "read",
        "times",
        "umount",
        "uname",
        "unlink",
        "wait",
        "waitpid",
        "write",
        "yield",
    ];

    for name in names.into_iter() {
        let args = vec![];
        let envs = vec![];

        let user_app = mm::load_user_app(String::from(name), args, envs)
            .expect("Testcase executable not found");

        let user_task = task::spawn_user_task(
            Arc::new(Mutex::new(user_app.aspace)),
            UspaceContext::new(user_app.entry.as_usize(), user_app.sp, 2333),
            user_app.break_pos,
        );

        let exit_code = user_task.join();
        info!("{name} exited with code: {:?}", exit_code);

        match exit_code {
            Some(n) if n != 0 => break,
            _ => {}
        }
    }
}
