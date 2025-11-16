use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{Ordering};
use tracing::{error, info, warn};
use crate::{ClientPacketHandler, PacketDiscriminants, ServerPacketHandler, PROTOCOL_VERSION};
use crate::{impl_packet_data, Packet, PacketData};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};
use crate::data::config::{ClientConfig, ServerConfig};
use crate::data::state::State;

#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Handshake {
    pub version: u32
}

impl_packet_data!(Handshake, ProtoHandshake);

impl ServerPacketHandler for Handshake {
    fn handle(&self, state: &Arc<State<ServerConfig>>, stream: &mut TcpStream) {
        info!("Client connected with version={}", &self.version);

        if self.version != PROTOCOL_VERSION {
            handle_mismatch(self.version);
        }
        
        let server_handshake = Handshake {
            version: PROTOCOL_VERSION
        };
        
        if Packet::send(stream, server_handshake, &state.aes_cipher).is_err() {
            warn!("Failed to send handshake to client");
            warn!("Disconnecting client from server forcibly");
            
            if stream.shutdown(Shutdown::Both).is_err() {
                error!("Failed to shutdown client stream during handshake");
            }
        }
    }
}

impl ClientPacketHandler for Handshake {
    fn handle(&self, state: &Arc<State<ClientConfig>>, stream: &mut TcpStream) {
        info!("Server connected with version={}", &self.version);

        if self.version != PROTOCOL_VERSION {
            handle_mismatch(self.version);

            if stream.shutdown(Shutdown::Both).is_err() {
                error!("Failed to shutdown stream during handshake");
            }

            state.running.store(false, Ordering::SeqCst);
            info!("Shutting down gracefully...");
        }
    }
}

fn handle_mismatch(self_version: u32) {
    warn!("Server/Client version mismatch c={} s={}", &self_version, PROTOCOL_VERSION);

    if &self_version > &PROTOCOL_VERSION {
        warn!("Server's protocol is newer, please update your client");
    } else {
        warn!("Client protocol is newer, please downgrade your client or update the remote server");
    }
}