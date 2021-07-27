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

use plic::Plic;
use rv_plic::Priority;
use uart::UART;

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

    logger::init();
    debug!("[kernel] Hello, world!");
    mm::init();
    mm::remap_test();
    uart::init();
    task::add_initproc();
    println!("initproc added to task manager!");
    trap::init();
    timer::set_next_trigger();
    loader::list_apps();

    Plic::set_threshold(1, Priority::any());
    Plic::enable(1, 10);
    Plic::set_priority(9, Priority::lowest());
    Plic::set_priority(10, Priority::lowest());
    println_uart!("uart print test");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
