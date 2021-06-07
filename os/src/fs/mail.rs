use alloc::{
    collections::VecDeque,
    sync::{Arc, Weak},
};
use spin::Mutex;

use crate::mm::UserBuffer;
use crate::task::suspend_current_and_run_next;

use super::File;

const MAIL_BUFFER_SIZE: usize = 256;
const MAILBOX_SIZE: usize = 16;

pub struct MailBox {
    inner: Mutex<MailBoxInner>,
}

pub struct MailBoxInner {
    mails: VecDeque<Arc<Mutex<MailRingBuffer>>>,
}

impl MailBox {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MailBoxInner {
                mails: VecDeque::new(),
            }),
        }
    }

    pub fn create_socket(&self) -> Arc<Socket> {
        debug!("create socket");
        let buffer = Arc::new(Mutex::new(MailRingBuffer::new()));
        let write_end = Arc::new(Socket::write_end_with_buffer(buffer.clone()));
        buffer.lock().set_write_end(&write_end);
        self.inner.lock().mails.push_back(buffer);
        write_end
    }

    pub fn is_empty(&self) -> bool {
        self.inner.lock().mails.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.inner.lock().mails.len() >= MAILBOX_SIZE
    }
}

impl File for MailBox {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize> {
        let mut inner = self.inner.lock();
        match inner.mails.front() {
            Some(mail) => {
                let mut buf_iter = buf.into_iter();
                let mut read_size: usize = 0;

                loop {
                    let mut ring_buffer = mail.lock();
                    let loop_read = ring_buffer.available_read();
                    if loop_read == 0 {
                        if ring_buffer.all_write_ends_closed() {
                            drop(ring_buffer);
                            inner.mails.pop_front();
                            return Ok(read_size);
                        }
                        drop(ring_buffer);
                        suspend_current_and_run_next();
                        continue;
                    }

                    for _ in 0..loop_read {
                        if let Some(byte_ref) = buf_iter.next() {
                            unsafe {
                                *byte_ref = ring_buffer.read_byte();
                            }
                            read_size += 1;
                        } else {
                            drop(ring_buffer);
                            inner.mails.pop_front();
                            return Ok(read_size);
                        }
                    }

                    drop(ring_buffer);
                    inner.mails.pop_front();
                    return Ok(read_size);
                }
            }
            None => Err(-1),
        }
    }

    fn write(&self, _buf: UserBuffer) -> Result<usize, isize> {
        Err(-1)
    }
}

pub struct Socket {
    writable: bool,
    mail: Arc<Mutex<MailRingBuffer>>,
}

impl Socket {
    pub fn write_end_with_buffer(buffer: Arc<Mutex<MailRingBuffer>>) -> Self {
        Self {
            writable: true,
            mail: buffer,
        }
    }
}

impl File for Socket {
    fn read(&self, _buf: UserBuffer) -> Result<usize, isize> {
        Err(-1)
    }

    fn write(&self, buf: UserBuffer) -> Result<usize, isize> {
        debug!("socket try to write");
        assert_eq!(self.writable, true);
        let mut buf_iter = buf.into_iter();
        let mut write_size: usize = 0;

        loop {
            let mut ring_buffer = self.mail.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                suspend_current_and_run_next();
                continue;
            }

            for _ in 0..loop_write {
                if let Some(byte_ref) = buf_iter.next() {
                    ring_buffer.write_byte(unsafe { *byte_ref });
                    write_size += 1;
                } else {
                    return Ok(write_size);
                }
            }

            return Ok(write_size);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum MailBufferStatus {
    FULL,
    EMPTY,
    NORMAL,
}

pub struct MailRingBuffer {
    arr: [u8; MAIL_BUFFER_SIZE],
    head: usize,
    tail: usize,
    status: MailBufferStatus,
    write_end: Option<Weak<Socket>>,
}

impl MailRingBuffer {
    pub fn new() -> Self {
        Self {
            arr: [0; MAIL_BUFFER_SIZE],
            head: 0,
            tail: 0,
            status: MailBufferStatus::EMPTY,
            write_end: None,
        }
    }

    pub fn set_write_end(&mut self, write_end: &Arc<Socket>) {
        self.write_end = Some(Arc::downgrade(write_end))
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.status = MailBufferStatus::NORMAL;
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % MAIL_BUFFER_SIZE;
        if self.tail == self.head {
            self.status = MailBufferStatus::FULL;
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        self.status = MailBufferStatus::NORMAL;
        let c = self.arr[self.head];
        self.head = (self.head + 1) % MAIL_BUFFER_SIZE;
        if self.head == self.tail {
            self.status = MailBufferStatus::EMPTY;
        }
        c
    }

    pub fn available_read(&self) -> usize {
        debug!("available_read {:?}", self.status);
        if self.status == MailBufferStatus::EMPTY {
            0
        } else {
            if self.tail > self.head {
                self.tail - self.head
            } else {
                self.tail + MAIL_BUFFER_SIZE - self.head
            }
        }
    }

    pub fn available_write(&self) -> usize {
        // debug!("available_write {:?}", self.status);
        if self.status == MailBufferStatus::FULL {
            0
        } else {
            MAIL_BUFFER_SIZE - self.available_read()
        }
    }

    pub fn all_write_ends_closed(&self) -> bool {
        self.write_end.as_ref().unwrap().upgrade().is_none()
    }
}
