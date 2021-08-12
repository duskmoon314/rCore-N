use super::File;
use crate::console_blog::pop_stdin;
use crate::mm::UserBuffer;
use crate::print;

pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn read(&self, mut user_buf: UserBuffer) -> Result<usize, isize> {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let c = pop_stdin();
        // c = console_getchar();

        let ch = c as u8;
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        Ok(1)
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
