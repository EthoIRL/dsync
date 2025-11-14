use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use crate::{ClientPacketHandler, PacketDiscriminants, ServerPacketHandler};
use crate::{impl_packet_data, Packet, PacketData};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};
use crate::state::State;

#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Add {
    pub data: u8,
    pub test: [u8; 2]
}

impl_packet_data!(Add, ObjectAdd);

impl ServerPacketHandler for Add {
    fn handle<T: Serialize + for<'a> Deserialize<'a> + Default + Send + Sync + 'static>(&self, state: &Arc<State<T>>) {
        info!("Hello handle!");
        info!("{:#?}", self);
        
        info!("{:#?}", self.id());
    }
}

impl ClientPacketHandler for Add {
    fn handle<T: Serialize + for<'a> Deserialize<'a> + Default + Send + Sync + 'static>(&self, state: &Arc<State<T>>) {
        warn!("Client received add packet, this should not happen!");
    }
}