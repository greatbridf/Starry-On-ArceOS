use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use axerrno::AxResult;
use axhal::{
    paging::MappingFlags,
    trap::{register_trap_handler, PAGE_FAULT},
};
use axmm::AddrSpace;
use axstd::{fs, io::Read};
use axtask::TaskExtRef;
use memory_addr::{MemoryAddr, VirtAddr};

use crate::{config, loader};

pub struct UserApp {
    /// The entry point of the user app.
    pub entry: VirtAddr,
    /// The top of the user stack.
    pub sp: VirtAddr,
    /// The program break position.
    pub break_pos: VirtAddr,
    /// The address space of the user app.
    pub aspace: AddrSpace,
}

/// Load a user app.
pub fn load_user_app(name: String, args: Vec<String>, envs: Vec<String>) -> AxResult<UserApp> {
    // TODO: Check shebang.
    if name.ends_with(".sh") {
        let args = [vec![String::from("busybox"), String::from("sh")], args].concat();
        return load_user_app(String::from("busybox"), args, envs);
    }

    let mut elf_data = vec![];
    fs::File::open(&name)?.read_to_end(&mut elf_data)?;

    let mut uspace = axmm::new_user_aspace(
        VirtAddr::from_usize(config::USER_SPACE_BASE),
        config::USER_SPACE_SIZE,
    )?;
    let elf_info = loader::load_elf(&elf_data, uspace.base());
    for segment in elf_info.segments.iter() {
        debug!(
            "Mapping ELF segment: [{:#x?}, {:#x?}) flags: {:#x?}",
            segment.start_vaddr,
            segment.start_vaddr + segment.size,
            segment.flags
        );
        uspace.map_alloc(segment.start_vaddr, segment.size, segment.flags, true)?;

        if segment.data.is_empty() {
            continue;
        }

        uspace.write(segment.start_vaddr + segment.offset, segment.data)?;

        // TDOO: flush the I-cache
    }

    let break_pos = elf_info
        .segments
        .iter()
        .fold(VirtAddr::from_usize(0), |cur, seg| {
            let end = seg.start_vaddr + seg.size;
            let end = end.align_up_4k();

            cur.max(end)
        });

    // The user stack is divided into two parts:
    // `ustack_start` -> `ustack_pointer`: It is the stack space that users actually read and write.
    // `ustack_pointer` -> `ustack_end`: It is the space that contains the arguments, environment variables and auxv passed to the app.
    //  When the app starts running, the stack pointer points to `ustack_pointer`.
    let ustack_end = VirtAddr::from_usize(config::USER_STACK_TOP);
    let ustack_size = config::USER_STACK_SIZE;
    let ustack_start = ustack_end - ustack_size;
    debug!(
        "Mapping user stack: {:#x?} -> {:#x?}",
        ustack_start, ustack_end
    );
    // FIXME: Add more arguments and environment variables
    let (stack_data, ustack_pointer) = kernel_elf_parser::get_app_stack_region(
        core::slice::from_ref(&name),
        &[],
        &elf_info.auxv,
        ustack_start,
        ustack_size,
    );
    uspace.map_alloc(
        ustack_start,
        ustack_size,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        true,
    )?;

    uspace.write(VirtAddr::from_usize(ustack_pointer), stack_data.as_slice())?;
    Ok(UserApp {
        entry: elf_info.entry,
        sp: VirtAddr::from(ustack_pointer),
        break_pos,
        aspace: uspace,
    })
}

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(vaddr: VirtAddr, access_flags: MappingFlags, is_user: bool) -> bool {
    if is_user {
        if !axtask::current()
            .task_ext()
            .aspace
            .lock()
            .handle_page_fault(vaddr, access_flags)
        {
            warn!(
                "{}: segmentation fault at {:#x}, exit!",
                axtask::current().id_name(),
                vaddr
            );
            axtask::exit(-1);
        }
        true
    } else {
        false
    }
}
