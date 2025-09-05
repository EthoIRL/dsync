use std::net::TcpStream;
use std::sync::Arc;
use redb::Database;
use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};

pub struct ObjectAdd;

impl GenericHandler for ObjectAdd {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {

        todo!()
    }
}