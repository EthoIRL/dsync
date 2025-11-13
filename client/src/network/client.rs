use crate::state::ClientState;
use proto::{header, Packet};
use std::error::Error;
use std::net::{IpAddr, TcpStream};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use tracing::warn;
use std::time::Duration;

pub fn connect(ip: IpAddr, port: u16, client_state: Arc<ClientState>) -> Result<TcpStream, Box<dyn Error>> {
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

pub trait ClientPacketHandler {
    fn handle(&self, state: &Arc<ClientState>);
}

pub fn client_listener(mut stream: TcpStream, client_state: Arc<ClientState>) {
    let mut packet_id: [u8; 1] = [0u8; 1];
    let mut packet_length_buffer: [u8; 4] = [0u8; 4];

    loop {
        if !client_state.running.load(Ordering::Relaxed) {
            info!("Network listener shutdown");
            return;
        }

        match header::get_packet(&mut stream, &mut packet_id, &mut packet_length_buffer) {
            Ok(packet_header) => {
                match Packet::parse(packet_header.id, &packet_header.data) {
                    Err(err) => {
                        warn!("Failed to parse packet ({})", err);
                    },
                    Ok(packet) => {
                        match &packet {
                            Packet::ObjectAdd(add) => add.handle(&client_state),
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
                warn!("Disconnected from remote server");
                return;
            }
        }
    }
}