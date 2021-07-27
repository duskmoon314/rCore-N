mod context;
mod usertrap;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::plic;
use crate::sbi::set_timer;
use crate::syscall::syscall;
use crate::task::{
    current_task, current_trap_cx, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::{set_next_trigger, TIMER_MAP};
use riscv::asm::ebreak;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sepc, sideleg, sie, sip, sstatus, stval, stvec, time,
};

global_asm!(include_str!("trap.asm"));

pub fn init() {
    unsafe {
        sie::set_stimer();
        sie::set_sext();
        sie::set_ssoft();
        sideleg::set_usoft();
        sideleg::set_uext();
        sideleg::set_utimer();
    }
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        extern "C" {
            fn kernelvec();
        }
        stvec::write(kernelvec as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get system call return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            error!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application, core dumped.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            let current_time = time::read();
            let mut timer_map = TIMER_MAP.lock();
            while let Some((_, pid)) = timer_map.pop_first() {
                if let Some((next_time, _)) = timer_map.first_key_value() {
                    if *next_time < current_time {
                        continue;
                    } else {
                        set_timer(*next_time);
                    }
                }
                drop(timer_map);
                if pid == 0 {
                    set_next_trigger();
                    suspend_current_and_run_next();
                } else if pid == current_task().unwrap().pid.0 {
                    unsafe {
                        sip::set_utimer();
                    }
                } else {
                    let _ = push_trap_record(
                        pid,
                        UserTrapRecord {
                            cause: 4,
                            message: current_time,
                        },
                    );
                }
                break;
            }
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            // debug!("Supervisor External");
            plic::handle_external_interrupt();
        }
        _ => {
            error!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    current_task()
        .unwrap()
        .acquire_inner_lock()
        .restore_user_trap_info();
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        llvm_asm!("fence.i" :::: "volatile");
        llvm_asm!("jr $0" :: "r"(restore_va), "{a0}"(trap_cx_ptr), "{a1}"(user_satp) :: "volatile");
    }
    panic!("Unreachable in back_to_user!");
}

#[no_mangle]
pub extern "C" fn trap_from_kernel() {
    unsafe {
        ebreak();
    }
    let scause = scause::read();
    let stval = stval::read();
    let _sepc = sepc::read();
    let _sstatus = sstatus::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            // unsafe {
            //     ebreak();
            // }
            suspend_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            // debug!("Supervisor External");
            unsafe {
                ebreak();
            }
            plic::handle_external_interrupt();
        }
        _ => {
            error!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
            panic!("a trap {:?} from kernel!", scause::read().cause());
        }
    }
}

pub use context::TrapContext;
pub use usertrap::{
    push_trap_record, UserTrapError, UserTrapInfo, UserTrapRecord, USER_EXT_INT_MAP,
};
