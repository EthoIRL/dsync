use std::{io, thread};
use std::error::Error;
use std::net::{Ipv4Addr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn connect(ip: Ipv4Addr, port: u16, application_running: Arc<AtomicBool>) -> io::Result<TcpStream> {
    let stream = TcpStream::connect((ip, port))?;

    let application_running = application_running.clone();
    let stream_reader = stream.try_clone()?;
    thread::spawn(move || {
        if let Err(err) = master_listener(stream_reader, application_running) {
            eprintln!("[*] [DSYNC] Unknown error occurred during master listening, (Error: {})", err);
        }
    });

    Ok(stream)
}

pub fn master_listener(mut stream: TcpStream, application_running: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    while application_running.load(Ordering::SeqCst) {

    }

    Ok(())
}