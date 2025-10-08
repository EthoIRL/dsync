use std::sync::Arc;
use tracing::warn;
use proto::Add;
use crate::network::server::ServerPacketHandler;
use crate::state::ServerState;

impl ServerPacketHandler for Add {
    fn handle(&self, state: &Arc<ServerState>) {
    }
}