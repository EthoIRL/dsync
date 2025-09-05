use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

mod network;

pub mod proto {
    pub mod comms {
        include!(concat!(env!("OUT_DIR"), "/comms.rs"));
    }
    pub mod constant {
        include!(concat!(env!("OUT_DIR"), "/constant.rs"));
    }
}

fn main() {
    println!("[*] [DSYNC] - [0.1.0] - [ETHO] [*]");

    let application_running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&application_running);

    ctrlc::set_handler(move || {
        running_clone.store(false, Ordering::SeqCst);
        println!("[*] [DSYNC] Server shutting down gracefully...");
    }).expect("[*] [DSYNC] Error setting Ctrl-C handler");

    while application_running.load(Ordering::SeqCst) {
    }
}
