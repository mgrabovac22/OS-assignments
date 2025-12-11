use libc::{c_long, c_char};
use std::mem;

#[repr(C)]
pub struct Message {
    pub mtype: c_long,
    pub mtext: [c_char; 1],
}

pub const EOF_CHAR: c_char = 0;
pub const MSG_KEY_VAR: &str = "MSG_KEY";
pub const MSG_SIZE: usize = mem::size_of::<[c_char; 1]>();