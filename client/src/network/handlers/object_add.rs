use std::sync::Arc;
use tracing::warn;
use proto::Add;
use crate::network::client::ClientPacketHandler;
use crate::state::ClientState;

impl ClientPacketHandler for Add {
    fn handle(&self, state: &Arc<ClientState>) {
        warn!("Client received add packet, this should not happen!");
    }
}