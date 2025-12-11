use std::env;
use std::process::exit;
use libc::{c_long, c_char};
use std::mem;

#[path = "../common.rs"] 
mod common;
use common::{Message, EOF_CHAR, MSG_KEY_VAR};

unsafe extern "C" {
    fn msgget(key: i32, msgflg: i32) -> i32;
    fn msgsnd(msqid: i32, msgp: *const Message, msgsz: usize, msgflg: i32) -> i32;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <string> <id>", args[0]);
        exit(1);
    }

    let text_to_send = &args[1];
    let raw_id = &args[2];
    
    let producer_id: c_long = match raw_id.parse() {
        Ok(id) if id > 0 => id,
        _ => {
            eprintln!("ERROR: Invalid producer ID: {}", raw_id);
            exit(1);
        }
    };
    
    println!("[Proizvodac] id: {}", producer_id);

    let key_str = env::var(MSG_KEY_VAR).unwrap_or_else(|_| {
        eprintln!("ERROR: Environment variable {} not found.", MSG_KEY_VAR);
        exit(1);
    });
    let key: i32 = key_str.parse().unwrap_or_else(|_| {
        eprintln!("ERROR: Invalid key value in {}.", MSG_KEY_VAR);
        exit(1);
    });
    
    let msqid = unsafe { msgget(key, 0) };
    if msqid < 0 {
        eprintln!("ERROR: Failed to get message queue ID (msgget).");
        exit(1);
    }

    let msg_size = mem::size_of::<[c_char; 1]>();
    
    for character in text_to_send.chars() {
        let message = Message {
            mtype: producer_id,
            mtext: [character as c_char; 1],
        };
        
        let result = unsafe {
            msgsnd(msqid, &message, msg_size, 0)
        };

        if result < 0 {
            eprintln!("ERROR: Failed to send message (msgsnd).");
            exit(1);
        }
    }

    let eof_message = Message {
        mtype: producer_id,
        mtext: [EOF_CHAR; 1],
    };
    
    let result = unsafe {
        msgsnd(msqid, &eof_message, msg_size, 0)
    };
    
    if result < 0 {
        eprintln!("ERROR: Failed to send EOF message (msgsnd).");
        exit(1);
    }

    println!("[Proizvodac] (id={}) zavrsio s radom!", producer_id);
}