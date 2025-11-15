use std::sync::Arc;
use tracing::{info, warn};
use crate::{ClientPacketHandler, PacketDiscriminants, ServerPacketHandler};
use crate::{impl_packet_data, Packet, PacketData};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};
use crate::data::config::{ClientConfig, ServerConfig};
use crate::data::state::State;

#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Add {
    pub data: u8,
    pub test: [u8; 2]
}

impl_packet_data!(Add, ObjectAdd);

impl ServerPacketHandler for Add {
    fn handle(&self, state: &Arc<State<ServerConfig>>) {
        info!("Hello handle!");
        info!("{:#?}", self);
        
        info!("{:#?}", self.id());
    }
}

impl ClientPacketHandler for Add {
    fn handle(&self, state: &Arc<State<ClientConfig>>) {
        warn!("Client received add packet, this should not happen!");
    }
}