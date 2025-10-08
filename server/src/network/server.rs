use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use tracing::{info, warn};
use proto::{header, Packet};
use crate::state::ServerState;

pub fn start_listening(ip: IpAddr, port: u16, server_state: Arc<ServerState>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind((ip, port))?;
    listener.set_nonblocking(true)?;

    loop {
        if !server_state.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let server_state = server_state.clone();
                thread::spawn(move || {
                    handle_client(stream, &server_state);
                });
            },
            Err(err) => {
                if err.kind() == ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                warn!("Error accepting connection ({})", err);
            }
        }
    }
}

pub trait ServerPacketHandler {
    fn handle(&self, state: &Arc<ServerState>);
}


fn handle_client(mut stream: TcpStream, server_state: &Arc<ServerState>) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    let peer_address = match stream.peer_addr() {
        Ok(addr) => addr.ip(),
        Err(err) => {
            warn!("Failed to get peer_addr, closing connection. ({})", err);
            return
        },
    };

    info!("Remote client connected [{}]", peer_address);

    match header::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
        Ok(packet_header) => {
            match Packet::parse(packet_header.id, &packet_header.data) {
                Err(err) => {
                    warn!("Failed to parse packet ({})", err);
                },
                Ok(packet) => {
                    match &packet {
                        Packet::ObjectAdd(add) => add.handle(&server_state),
                    }
                }
            }
        },
        Err(err) => {
            warn!("Failed to get packet ({})", err);
        }
    }
}