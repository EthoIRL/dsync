use proto::Add;
use crate::network::client::ClientPacketHandler;

impl ClientPacketHandler for Add {
    fn handle(&self) {
        println!("Handling ObjectAdd: {:?}", self);
    }
}