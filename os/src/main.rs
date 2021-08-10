#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(map_first_last)]

extern crate alloc;
extern crate rv_plic;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

#[macro_use]
mod console;
mod config;
mod console_blog;
mod fs;
mod lang_items;
mod loader;
mod logger;
mod mm;
mod plic;
mod sbi;
mod syscall;
mod task;
mod timer;
mod trap;
#[macro_use]
mod uart;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[allow(dead_code)]
fn hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        asm!("mv {}, tp", out(reg) hart_id);
    }
    hart_id
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();

    // enable simulation log on rocket core
    #[cfg(feature = "board_lrv")]
    unsafe {
        asm!("csrwi 0x800, 1");
    }

    mm::init();
    trap::init();
    plic::init();
    uart::init();
    logger::init();
    debug!("[kernel] Hello, world!");
    mm::remap_test();
    task::add_initproc();
    println!("initproc added to task manager!");
    timer::set_next_trigger();
    loader::list_apps();

    println_uart!("uart print test");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
