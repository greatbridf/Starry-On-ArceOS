use axhal::paging::MappingFlags;
use axtask::TaskExtRef;
use memory_addr::{MemoryAddr, VirtAddr, VirtAddrRange};

pub(crate) fn sys_brk(new_break: VirtAddr) -> VirtAddr {
    let current = axtask::current();
    let task_ext = current.task_ext();

    let mut break_pos = task_ext.break_pos.lock();
    let break_start = task_ext.break_start;
    let current_break = *break_pos;

    let new_break = new_break.align_up_4k();
    if new_break <= current_break {
        return current_break;
    }

    let mut aspace = task_ext.aspace.lock();

    let new_range = VirtAddrRange::new(current_break, new_break);
    if aspace.overlap(new_range) {
        return current_break;
    }

    aspace.map_alloc(
        current_break,
        new_range.size(),
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        false,
    );

    *break_pos = new_break;
    new_break
}
