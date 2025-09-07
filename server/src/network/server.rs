use crate::network::handlers::object_add::{Object};
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::constant::PacketKind;
use std::collections::HashMap;
use std::error::Error;
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};
use std::io::ErrorKind;
use std::time::Duration;
use redb::Database;
use crate::config::Config;
use crate::network::handlers::object_chunk_response::ObjectChunkResponse;
use crate::network::handlers::object_status_response::ObjectStatusResponse;
use crate::network::handlers::object_sync::ObjectSync;
use crate::network::handlers::object_sync_response::ObjectSyncResponse;
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::AddResponse;

pub fn start_listening(ip: Ipv4Addr, port: u16, application_running: Arc<AtomicBool>, config: Arc<Config>, database: Arc<Database>) -> io::Result<()> {
    let listener = TcpListener::bind((ip, port))?;
    listener.set_nonblocking(true)?;

    println!("[*] [DSYNC] Listening on {}:{}", ip, port);

    while application_running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let config = config.clone();
                let database = database.clone();
                let application_running = application_running.clone();

                thread::spawn(move || {
                    handle_client(application_running, stream, config, database);
                });
            },
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                eprintln!("[*] [DSYNC] Error accepting client connection: {}", e);
            }
        }
    }
    
    Ok(())
}

type GenericHandlerType = fn(&mut TcpStream, GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>>;

fn handle_client(application_running: Arc<AtomicBool>, mut stream: TcpStream, config: Arc<Config>, database: Arc<Database>) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    let peer_address = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(_) => {
            eprintln!("[*] [DSYNC] Failed to get peer_addr, closing connection.");
            return
        },
    };

    println!("[*] [DSYNC] Remote client connected [{}]", peer_address);

    let mut packet_handlers: HashMap<u8, GenericHandlerType> = HashMap::new();
    packet_handlers.insert(PacketKind::ObjectAdd as u8, Object::handle);
    packet_handlers.insert(PacketKind::ObjectSync as u8, ObjectSync::handle);
    packet_handlers.insert(PacketKind::ObjectStatusResponse as u8, ObjectStatusResponse::handle);
    packet_handlers.insert(PacketKind::ObjectSyncResponse as u8, ObjectSyncResponse::handle);
    packet_handlers.insert(PacketKind::ObjectChunkResponse as u8, ObjectChunkResponse::handle);

    while application_running.load(Ordering::SeqCst) {
        match packet::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet) => {
                match PacketKind::try_from(packet.id as i32) {
                    Ok(packet_kind) => {
                        println!("[*] [DSYNC] Handling: {:#?}", packet_kind);
                        if let Err(err) = handle_generic_packet(&mut stream, packet_kind, packet, &packet_handlers, &config, &database) {
                            eprintln!("[*] [DSYNC] Failed to handle packet (Error: {}, Client: {}, Id: {})", err, peer_address, packet_id[0]);
                        }
                    },
                    Err(_) => {
                        eprintln!("[*] [DSYNC] Unknown packet found in data stream, (ID: {}, Client: {})", packet.id, peer_address);
                    }
                }
            },
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                if err.kind() == ErrorKind::ConnectionReset {
                    return;
                }

                eprintln!("[*] [DSYNC] Failed to get packet, ({}, Client: {})", err, peer_address);
                return;
            }
        }
    }
}

fn handle_generic_packet(
    stream: &mut TcpStream,
    packet_kind: PacketKind,
    packet: GenericPacket,
    packet_handlers: &HashMap<u8, GenericHandlerType>,
    config: &Arc<Config>,
    database: &Arc<Database>
) -> Result<(), Box<dyn Error>> {
    if let Some(handle) = packet_handlers.get(&packet.id) {
        if let Err(err) = handle(stream, packet, config, database) {
            match packet_kind {
                PacketKind::ObjectAdd => {
                    println!("[*] [DSYNC] Handling object add error... ({})", err);

                    let add_response = AddResponse {
                        object_id: Vec::new(),
                        path: String::new(),
                        success: false,
                        error: Some(AddError::Unknown as i32)
                    };

                    packet::send_packet(stream, &mut [PacketKind::ObjectAddResponse as u8], add_response)?;
                }
                _ => {
                    eprintln!("[*] [DSYNC] Can't handle error of packet, (FIX ME!) (Error: {}, Id: {:?})", err, packet_kind);
                }
            }
        }
    }

    Ok(())
}