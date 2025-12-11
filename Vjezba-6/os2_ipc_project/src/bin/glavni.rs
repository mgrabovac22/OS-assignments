#[path = "../common.rs"] 
mod common;
use common::MSG_KEY_VAR;

use std::env;
use std::process::{Command, exit};
use libc::getuid; 

unsafe extern "C" {
    fn msgget(key: i32, msgflg: i32) -> i32;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 || (args.len() - 1) % 2 != 0 {
        eprintln!("Usage: {} <string1> <id1> [<string2> <id2> ...]", args[0]);
        exit(1);
    }

    let key = unsafe { getuid() } as i32; 

    let msqid = unsafe { msgget(key, libc::IPC_CREAT | 0o600) };

    if msqid < 0 {
        eprintln!("ERROR: Failed to create message queue ID (msgget).");
        exit(1);
    }
    
    println!("[S] Nadjena varijabla okoline {}", key);
    unsafe {
        env::set_var(MSG_KEY_VAR, key.to_string());
    }
    
    let mut producer_args: Vec<(String, String)> = Vec::new();
    let num_producers = (args.len() - 1) / 2;

    for i in 0..num_producers {
        let text = args[2 * i + 1].clone();
        let id = args[2 * i + 2].clone();
        producer_args.push((text, id));
    }

    let mut children = Vec::new();

    match Command::new("./target/debug/potrosac").spawn() {
        Ok(child) => {
            println!("[R] Nadjena varijabla okoline {}", key); 
            children.push(child);
        },
        Err(e) => eprintln!("ERROR: Failed to start potrosac: {}. Pokrenite 'cargo build' i provjerite putanje.", e),
    }

    for (text, id) in producer_args {
        match Command::new("./target/debug/proizvodac")
                      .arg(&text)
                      .arg(&id)
                      .spawn() {
            Ok(child) => children.push(child),
            Err(e) => eprintln!("ERROR: Failed to start proizvodac ({} {}): {}. Pokrenite 'cargo build' i provjerite putanje.", text, id, e),
        }
    }

    for mut child in children {
        let _ = child.wait();
    }
    
    println!("Glavni program zavrsio.");
}