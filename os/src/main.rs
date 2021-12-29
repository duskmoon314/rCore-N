#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(map_first_last)]
#![feature(map_try_insert)]

extern crate alloc;
extern crate rv_plic;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

use crate::{config::CPU_NUM, mm::init_kernel_space, sbi::send_ipi};

#[macro_use]
mod console;
mod config;
#[macro_use]
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

#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {
    if hart_id == 0 {
        clear_bss();
        logger::init();
        mm::init();
        debug!("[kernel {}] Hello, world!", hart_id);
        mm::remap_test();
        trap::init();
        plic::init();
        plic::init_hart(hart_id);
        uart::init();

        extern "C" {
            fn boot_stack();
            fn boot_stack_top();
        }

        debug!(
            "boot_stack {:#x} top {:#x}",
            boot_stack as usize, boot_stack_top as usize
        );

        debug!("trying to add initproc");
        task::add_initproc();
        debug!("initproc added to task manager!");

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }

        for i in 1..CPU_NUM {
            debug!("[kernel {}] Start {}", hart_id, i);
            let mask: usize = 1 << i;
            send_ipi(&mask as *const _ as usize);
        }
    } else {
        let hart_id = task::hart_id();

        init_kernel_space();

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }
        trap::init();
        plic::init_hart(hart_id);
    }

    println_hart!("Hello", hart_id);

    timer::set_next_trigger();

    if hart_id == 0 {
        loader::list_apps();
    }

    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
