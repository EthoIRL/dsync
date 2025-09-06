use crate::config::Config;
use crate::network::handlers::object_add_response::ObjectAddResponse;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::constant::PacketKind;
use redb::Database;
use std::collections::HashMap;
use std::error::Error;
use std::net::{Ipv4Addr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{io, thread};
use std::io::ErrorKind;
use crate::network::handlers::object_sync::ObjectSync;

pub fn connect(ip: Ipv4Addr, port: u16, application_running: Arc<AtomicBool>, config: Arc<Config>, database: Arc<Database>) -> io::Result<TcpStream> {
    let stream = TcpStream::connect((ip, port))?;

    let application_running = application_running.clone();
    let stream_reader = stream.try_clone()?;
    let config = Arc::clone(&config);
    let database = Arc::clone(&database);
    thread::spawn(move || {
        master_listener(stream_reader, application_running, config, database);
    });

    Ok(stream)
}

type GenericHandlerType = fn(&mut TcpStream, GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>>;

pub fn master_listener(mut stream: TcpStream, application_running: Arc<AtomicBool>, config: Arc<Config>, database: Arc<Database>) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    let mut packet_handlers: HashMap<u8, GenericHandlerType> = HashMap::new();
    packet_handlers.insert(PacketKind::ObjectAddResponse as u8, ObjectAddResponse::handle);
    packet_handlers.insert(PacketKind::ObjectSync as u8, ObjectSync::handle);

    while application_running.load(Ordering::SeqCst) {
        match packet::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet) => {
                match PacketKind::try_from(packet.id as i32) {
                    Ok(packet_kind) => {
                        println!("[*] [DSYNC] Handling: {:#?}", packet_kind);
                        if let Err(err) = handle_generic_packet(&mut stream, packet_kind, packet, &packet_handlers, &config, &database) {
                            eprintln!("[*] [DSYNC] Failed to handle packet (Error: {}, Id: {})", err, packet_id[0]);
                        }
                    },
                    Err(_) => {
                        eprintln!("[*] [DSYNC] Unknown packet found in data stream, (ID: {})", packet.id);
                    }
                }
            },
            Err(err) => {
                if err.kind() == ErrorKind::ConnectionAborted {
                    return;
                }

                eprintln!("[*] [DSYNC] Failed to get packet, ({})", err);
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
                _ => {
                    eprintln!("[*] [DSYNC] Can't handle error of packet, (FIX ME!) (Error: {}, Id: {:?})", err, packet_kind);
                }
            }
        }
    }

    Ok(())
}