use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::{io, thread};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::network::packet;
use crate::proto::constant::PacketKind;

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

        if let Ok((stream, _)) = listener.accept() {
            thread::spawn(move || {
                handle_client(stream);
            });
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

    loop {
        match packet::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet) => {
                match PacketKind::try_from(packet.id as i32) {
                    Ok(known_id) => {
                        //TODO: Handle incoming packets
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