use proto::{header, ClientPacketHandler, Packet};
use std::error::Error;
use std::io::ErrorKind;
use std::net::{IpAddr, TcpStream};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::BlockDecrypt;
use tracing::{error, info, warn};
use proto::data::config::ClientConfig;
use proto::data::state::State;

pub fn connect(ip: IpAddr, port: u16, client_state: Arc<State<ClientConfig>>) -> Result<TcpStream, Box<dyn Error>> {
    let stream = TcpStream::connect((ip, port))?;

    stream.set_nodelay(true)?;
    stream.set_nonblocking(true)?;

    let client_state = client_state.clone();
    let stream_reader = stream.try_clone()?;
    thread::spawn(move || {
        client_listener(stream_reader, client_state);
    });

    Ok(stream)
}

pub fn client_listener(mut stream: TcpStream, client_state: Arc<State<ClientConfig>>) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    loop {
        if !client_state.running.load(Ordering::Relaxed) {
            info!("Network listener shutdown");
            return;
        }

        match header::get(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet_header) => {
                let mut packet_data: Vec<u8> = packet_header.data;

                if let Some(cipher) = &client_state.aes_cipher {
                    packet_data = match cipher.decrypt_padded_vec::<Pkcs7>(&packet_data) {
                        Ok(data) => data,
                        Err(_) => {
                            error!("Failed to decrypt incoming remote data");
                            error!("Disconnected from remote server");
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
                            Packet::ObjectAdd(add) => add.handle(&client_state, &mut stream),
                            Packet::ProtoHandshake(handshake) => handshake.handle(&client_state, &mut stream)
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

                error!("Failed to get packet ({})", err);
                error!("Disconnected from remote server");
                return;
            }
        }
    }
}