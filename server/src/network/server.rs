use crate::network::handlers::object_add::ObjectAdd;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::constant::PacketKind;
use std::collections::HashMap;
use std::error::Error;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};

pub fn start_listening(ip: Ipv4Addr, port: u16, application_running: Arc<AtomicBool>) -> io::Result<()> {
    let listener = TcpListener::bind((ip, port))?;

    println!("[*] [DSYNC] Listening on {}:{}", ip, port);

    while application_running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            },
            Err(e) => {
                eprintln!("[*] [DSYNC] Error accepting client connection: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    let peer_address = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => {
            eprintln!("[*] [DSYNC] Failed to get peer_addr, closing connection.");
            return
        },
    };

    let mut packet_handlers: HashMap<u8, fn(&mut TcpStream, GenericPacket) -> Result<(), Box<dyn Error>>> = HashMap::new();
    packet_handlers.insert(0, ObjectAdd::handle);

    loop {
        match packet::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet) => {
                match PacketKind::try_from(packet.id as i32) {
                    Ok(packet_kind) => {
                        println!("[*] [DSYNC] Handling: {:#?}", packet_kind);

                        //TODO: Handle incoming packets
                        if let Err(err) = handle_generic_packet(&mut stream, packet, &packet_handlers) {
                            eprintln!("[*] [DSYNC] Failed to handle packet (Error: {}, Client: {}, Id: {})", err, peer_address, packet_id[0]);
                        }
                    },
                    Err(_) => {
                        eprintln!("[*] [DSYNC] Unknown packet found in data stream, (ID: {}, Client: {})", packet.id, peer_address);
                    }
                }
            },
            Err(err) => {
                eprintln!("[*] [DSYNC] Failed to get packet, ({}, Client: {})", err, peer_address);
                return;
            }
        }
    }
}

fn handle_generic_packet(stream: &mut TcpStream, packet: GenericPacket, packet_handlers: &HashMap<u8, fn(&mut TcpStream, GenericPacket) -> Result<(), Box<dyn Error>>>) -> Result<(), Box<dyn Error>> {
    if let Some(handle) = packet_handlers.get(&packet.id) {
        handle(stream, packet)?;
    }

    Err("".into())
}