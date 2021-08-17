use super::File;
use crate::mm::UserBuffer;
use crate::uart::{serial_getchar, serial_putchar};

pub struct Serial<const N: usize>;

impl<const N: usize> File for Serial<N> {
    fn read(&self, mut user_buf: UserBuffer) -> Result<usize, isize> {
        assert_eq!(user_buf.len(), 1);
        // busy loop
        let ch = serial_getchar(N);
        unsafe {
            user_buf.buffers[0].as_mut_ptr().write_volatile(ch);
        }
        Ok(1)
    }
    fn write(&self, user_buf: UserBuffer) -> Result<usize, isize> {
        for buffer in user_buf.buffers.iter() {
            for char in buffer.iter() {
                serial_putchar(N, *char);
            }
        }
        Ok(user_buf.len())
    }
}
