use embedded_hal::serial::Write;

use super::File;
use crate::mm::UserBuffer;
use crate::uart::{serial_getchar, BUFFERED_SERIAL};

pub struct Serial<const N: usize>;

impl<const N: usize> File for Serial<N> {
    fn read(&self, user_buf: UserBuffer) -> Result<usize, isize> {
        let mut read_cnt = 0;
        let mut buf_iter = user_buf.into_iter();
        while let Some(ptr) = buf_iter.next() {
            if let Ok(ch) = serial_getchar(N) {
                // debug!("Serial {} read: {}", N, ch);
                unsafe {
                    ptr.write_volatile(ch);
                }
                read_cnt += 1;
            } else {
                break;
            }
        }
        // debug!("Serial {} read cnt: {}", N, read_cnt);
        if read_cnt > 0 {
            Ok(read_cnt)
        } else {
            Err(-1)
        }
    }
    fn write(&self, user_buf: UserBuffer) -> Result<usize, isize> {
        let mut write_cnt = 0;
        let mut write_ok = true;
        let mut serial = BUFFERED_SERIAL[N].lock();
        for buffer in user_buf.buffers.iter() {
            for char in buffer.iter() {
                // debug!("Serial {} write: {}", N, *char);
                if let Ok(()) = serial.try_write(*char) {
                    write_cnt += 1;
                } else {
                    write_ok = false;
                    break;
                }
            }
            if !write_ok {
                break;
            }
        }
        if write_cnt > 0 {
            Ok(write_cnt)
        } else {
            Err(-1)
        }
    }
}
