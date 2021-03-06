use crate::task::hart_id;
use crate::{console::ANSICON, sbi::shutdown};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println_colorized!(
            "[kernel {}] Panicked at {}:{} {}",
            ANSICON::FgRed,
            ANSICON::BgDefault,
            hart_id(),
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println_colorized!(
            "[kernel {}] Panicked: {}",
            ANSICON::FgRed,
            ANSICON::BgDefault,
            hart_id(),
            info.message().unwrap()
        );
    }
    shutdown()
}
