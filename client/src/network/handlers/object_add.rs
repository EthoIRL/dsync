use std::sync::Arc;
use proto::Add;
use crate::network::client::ClientPacketHandler;
use crate::state::ClientState;

impl ClientPacketHandler for Add {
    fn handle(&self, state: &Arc<ClientState>) {
        println!("Handling ObjectAdd: {:?}", self);
    }
}