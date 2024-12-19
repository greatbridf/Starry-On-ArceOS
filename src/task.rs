use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;
use memory_addr::VirtAddr;

use axhal::arch::UspaceContext;
use axmm::AddrSpace;
use axsync::Mutex;
use axtask::{AxTaskRef, TaskExtRef, TaskInner};

/// Task extended data for the monolithic kernel.
pub struct TaskExt {
    /// The process ID.
    pub proc_id: usize,
    /// The clear thread tid field
    ///
    /// See <https://manpages.debian.org/unstable/manpages-dev/set_tid_address.2.en.html#clear_child_tid>
    ///
    /// When the thread exits, the kernel clears the word at this address if it is not NULL.
    clear_child_tid: AtomicU64,
    /// The user space context.
    pub uctx: UspaceContext,
    /// The virtual memory address space.
    pub aspace: Arc<Mutex<AddrSpace>>,
    /// The start position of the program break.
    pub break_start: VirtAddr,
    /// The current position of the program break.
    pub break_pos: Mutex<VirtAddr>,
}

impl TaskExt {
    pub const fn new(
        uctx: UspaceContext,
        aspace: Arc<Mutex<AddrSpace>>,
        break_start: VirtAddr,
    ) -> Self {
        Self {
            proc_id: 233,
            uctx,
            clear_child_tid: AtomicU64::new(0),
            aspace,
            break_start,
            break_pos: Mutex::new(break_start),
        }
    }

    pub(crate) fn clear_child_tid(&self) -> u64 {
        self.clear_child_tid
            .load(core::sync::atomic::Ordering::Relaxed)
    }

    pub(crate) fn set_clear_child_tid(&self, clear_child_tid: u64) {
        self.clear_child_tid
            .store(clear_child_tid, core::sync::atomic::Ordering::Relaxed);
    }
}

axtask::def_task_ext!(TaskExt);

pub fn spawn_user_task(
    aspace: Arc<Mutex<AddrSpace>>,
    uctx: UspaceContext,
    break_start: VirtAddr,
) -> AxTaskRef {
    let mut task = TaskInner::new(
        || {
            let curr = axtask::current();
            let kstack_top = curr.kernel_stack_top().unwrap();
            info!(
                "Enter user space: entry={:#x}, ustack={:#x}, kstack={:#x}",
                curr.task_ext().uctx.get_ip(),
                curr.task_ext().uctx.get_sp(),
                kstack_top,
            );
            unsafe { curr.task_ext().uctx.enter_uspace(kstack_top) };
        },
        "[usertask]".into(),
        crate::config::KERNEL_STACK_SIZE,
    );
    task.ctx_mut()
        .set_page_table_root(aspace.lock().page_table_root());
    task.init_task_ext(TaskExt::new(uctx, aspace, break_start));
    axtask::spawn_task(task)
}
