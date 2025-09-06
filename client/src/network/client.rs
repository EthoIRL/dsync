use std::{io, thread};
use std::collections::HashMap;
use std::error::Error;
use std::net::{Ipv4Addr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::network::handlers::object_add_response::ObjectAddResponse;
use crate::network::packet::{GenericHandler, GenericPacket};

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
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    let mut packet_handlers: HashMap<u8, fn(&mut TcpStream, GenericPacket) -> Result<(), Box<dyn Error>>> = HashMap::new();
    packet_handlers.insert(0, ObjectAddResponse::handle);

    while application_running.load(Ordering::SeqCst) {

    }

    Ok(())
}