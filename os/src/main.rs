#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(llvm_asm)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;
extern crate rv_plic;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

use rv_plic::Priority;
use plic::Plic;

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

#[macro_export]
macro_rules! print_uart
{
	($($args:tt)+) => ({
			use core::fmt::Write;
			let _ = write!(crate::uart::Uart::new(0x1000_0000), $($args)+);
			});
}
#[macro_export]
macro_rules! println_uart
{
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
			print_uart!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			print_uart!(concat!($fmt, "\r\n"), $($args)+)
			});
}

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    uart::Uart::new(0x10000000).init();
    logger::init();
    debug!("[kernel] Hello, world!");
    mm::init();
    mm::remap_test();
    task::add_initproc();
    println!("after initproc!");
    trap::init();
    trap::enable_timer_interrupt();
    trap::enable_external_interrupt();
    timer::set_next_trigger();
    loader::list_apps();

    Plic::set_threshold(1, Priority::any());
    Plic::enable(1, 10);
    Plic::set_priority(10, Priority::lowest());
    println_uart!("uart print test");
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
