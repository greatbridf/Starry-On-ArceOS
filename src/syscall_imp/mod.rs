mod fs;
mod mm;
mod task;
mod time;

use axerrno::{LinuxError, LinuxResult};
use axhal::{
    arch::TrapFrame,
    trap::{register_trap_handler, SYSCALL},
};
use memory_addr::VirtAddr;
use syscalls::Sysno;

use self::fs::*;
use self::mm::*;
use self::task::*;
use self::time::*;

/// Macro to generate syscall body
///
/// It will receive a function which return Result<_, LinuxError> and convert it to
/// the type which is specified by the caller.
#[macro_export]
macro_rules! syscall_body {
    ($fn: ident, $($stmt: tt)*) => {{
        #[allow(clippy::redundant_closure_call)]
        let res = (|| -> axerrno::LinuxResult<_> { $($stmt)* })();
        match res {
            Ok(_) | Err(axerrno::LinuxError::EAGAIN) => debug!(concat!(stringify!($fn), " => {:?}"),  res),
            Err(_) => info!(concat!(stringify!($fn), " => {:?}"), res),
        }
        match res {
            Ok(v) => v as _,
            Err(e) => {
                -e.code() as _
            }
        }
    }};
}

fn do_handle_syscall(tf: &TrapFrame, syscall_num: usize) -> LinuxResult<isize> {
    Ok(match Sysno::from(syscall_num as u32) {
        Sysno::read => sys_read(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _),
        Sysno::write => sys_write(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _),
        Sysno::mmap => sys_mmap(
            tf.arg0() as _,
            tf.arg1() as _,
            tf.arg2() as _,
            tf.arg3() as _,
            tf.arg4() as _,
            tf.arg5() as _,
        ) as _,
        Sysno::ioctl => sys_ioctl(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _) as _,
        Sysno::writev => sys_writev(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _),
        Sysno::sched_yield => sys_sched_yield() as isize,
        Sysno::nanosleep => sys_nanosleep(tf.arg0() as _, tf.arg1() as _) as _,
        Sysno::getpid => sys_getpid() as isize,
        Sysno::exit => sys_exit(tf.arg0() as _),
        #[cfg(target_arch = "x86_64")]
        Sysno::arch_prctl => sys_arch_prctl(tf.arg0() as _, tf.arg1() as _),
        Sysno::set_tid_address => sys_set_tid_address(tf.arg0() as _),
        Sysno::clock_gettime => sys_clock_gettime(tf.arg0() as _, tf.arg1() as _) as _,
        Sysno::exit_group => sys_exit_group(tf.arg0() as _),
        #[cfg(target_arch = "x86_64")]
        Sysno::clock_gettime => sys_clock_gettime(tf.arg0() as _, tf.arg1() as _) as _,
        #[cfg(not(target_arch = "x86_64"))]
        Sysno::clock_gettime64 => sys_clock_gettime(tf.arg0() as _, tf.arg1() as _) as _,
        // TODO: `clock_id` and `flags`
        Sysno::clock_nanosleep_time64 => sys_nanosleep(tf.arg2() as _, tf.arg3() as _) as _,
        #[cfg(not(target_arch = "x86_64"))]
        Sysno::clock_nanosleep_time64 => sys_nanosleep(tf.arg2() as _, tf.arg3() as _) as _,
        Sysno::brk => {
            let vaddr = VirtAddr::from_usize(tf.arg0());
            sys_brk(vaddr).as_usize() as isize
        }
        Sysno::mkdirat => sys_mkdirat(tf.arg0() as _, tf.arg1() as _, tf.arg2() as _) as _,
        Sysno::chdir => sys_chdir(tf.arg0() as _) as _,
        Sysno::getcwd => sys_getcwd(tf.arg0() as _, tf.arg1() as _) as _,
        _ => {
            warn!("Unimplemented syscall: {}", syscall_num);
            axtask::exit(LinuxError::ENOSYS as _)
        }
    })
}

#[register_trap_handler(SYSCALL)]
fn handle_syscall(tf: &TrapFrame, syscall_num: usize) -> isize {
    match do_handle_syscall(tf, syscall_num) {
        Ok(retval) => retval,
        Err(error) => -error.code() as isize,
    }
}
