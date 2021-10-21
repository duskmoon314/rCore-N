use super::File;
use crate::mm::UserBuffer;
use crate::print;
use crate::uart::{serial_getchar, serial_putchar};
use core::fmt::{self, Write};

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn read(&self, mut user_buf: UserBuffer) -> Result<usize, isize> {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        if let Ok(ch) = serial_getchar(0) {
            unsafe {
                user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
            }
            Ok(1)
        } else {
            Err(-1)
        }
    }
    fn write(&self, _user_buf: UserBuffer) -> Result<usize, isize> {
        panic!("Cannot write to stdin!");
    }
}

impl File for Stdout {
    fn read(&self, _user_buf: UserBuffer) -> Result<usize, isize> {
        panic!("Cannot read from stdout!");
    }
    fn write(&self, user_buf: UserBuffer) -> Result<usize, isize> {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        Ok(user_buf.len())
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            serial_putchar(0, c as u8);
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::fs::stdio::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::fs::stdio::print(format_args!(concat!($fmt, "\r\n") $(, $($arg)+)?));
    }
}
