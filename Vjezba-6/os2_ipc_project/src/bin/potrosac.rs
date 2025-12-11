use std::env;
use std::process::exit;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;
use libc::{c_long, IPC_NOWAIT}; 

#[path = "../common.rs"] 
mod common;
use common::{Message, EOF_CHAR, MSG_KEY_VAR, MSG_SIZE};

unsafe extern "C" {
    fn msgget(key: i32, msgflg: i32) -> i32;
    fn msgrcv(msqid: i32, msgp: *mut Message, msgsz: usize, msgtyp: c_long, msgflg: i32) -> isize;
    fn msgctl(msqid: i32, cmd: i32, buf: *mut ()) -> i32;
}

fn main() {
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
    
    println!("[R] Nadjena varijabla okoline {}", key);

    let mut received_strings: HashMap<c_long, String> = HashMap::new();
    let mut producers_finished: HashMap<c_long, bool> = HashMap::new();
    
    let msg_size = MSG_SIZE;
    let mut consecutive_empty_reads = 0;
    const MAX_EMPTY_READS: i32 = 10;
    
    println!("[Potrosac] pokrenut");
    
    loop {
        let mut message = Message { mtype: 0, mtext: [0; 1] };
        
        let bytes_received = unsafe {
            msgrcv(msqid, &mut message, msg_size, 0, IPC_NOWAIT) 
        };

        if bytes_received > 0 {
            consecutive_empty_reads = 0;

            let producer_id = message.mtype;
            let received_char = message.mtext[0];

            if received_char == EOF_CHAR {
                println!("[Proizvodac] (id={}) zavrsio s radom!", producer_id); 
                producers_finished.insert(producer_id, true);
                
                if let Some(s) = received_strings.get(&producer_id) {
                    println!("{}", s);
                }
            } else {
                let c = received_char as u8 as char;
                println!("[Potrosac] primio: {} od :{}", c, producer_id);
                
                received_strings.entry(producer_id)
                    .or_insert_with(String::new)
                    .push(c);
            }
        } else if bytes_received == -1 {
            consecutive_empty_reads += 1;
            println!("[Potrosac] cekam ({})", consecutive_empty_reads);
            
            if consecutive_empty_reads >= MAX_EMPTY_READS {
                break; 
            }
            
            sleep(Duration::from_secs(1));
        } else {
            eprintln!("ERROR: Message receive error: {}", bytes_received);
            break;
        }
    }

    let result = unsafe { 
        msgctl(msqid, libc::IPC_RMID, std::ptr::null_mut()) 
    };

    if result < 0 {
        eprintln!("ERROR: Failed to destroy message queue (msgctl IPC_RMID).");
    }

    println!("[Potrosac] zavrsava s radom");
}