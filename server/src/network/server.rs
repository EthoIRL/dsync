use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::BlockDecrypt;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use proto::{header, Packet, ServerPacketHandler};
use proto::state::State;

pub fn start_listening<T: Serialize + for<'a> Deserialize<'a> + Default + Send + Sync + 'static>(ip: IpAddr, port: u16, server_state: Arc<State<T>>) -> Result<(), Box<dyn Error>> {
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

fn handle_client<T: Serialize + for<'a> Deserialize<'a> + Default + Send + Sync + 'static>(mut stream: TcpStream, server_state: &Arc<State<T>>) {
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

    loop {
        if !server_state.running.load(Ordering::Relaxed) {
            return;
        }

        match header::get(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet_header) => {
                let mut packet_data: Vec<u8> = packet_header.data;

                if let Some(cipher) = &server_state.aes_cipher {
                    packet_data = match cipher.decrypt_padded_vec::<Pkcs7>(&packet_data) {
                        Ok(data) => data,
                        Err(_) => {
                            error!("Failed to decrypt incoming client data");
                            error!("Client forceable disconnected");
                            return;
                        }
                    }
                }

                match Packet::parse(packet_header.id, &packet_data) {
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
                if err.kind() == ErrorKind::WouldBlock {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                if err.kind() == ErrorKind::ConnectionReset {
                    return;
                }

                warn!("Failed to get packet ({})", err);
                warn!("Client forceable disconnected");
                return;
            }
        }
    }
}