mod mail;
mod pipe;
mod stdio;

use crate::mm::UserBuffer;

pub use mail::{MailBox, Socket};
pub trait File: Send + Sync {
    fn read(&self, buf: UserBuffer) -> Result<usize, isize>;
    fn write(&self, buf: UserBuffer) -> Result<usize, isize>;
}

pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
