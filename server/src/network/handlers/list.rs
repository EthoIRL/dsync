use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use redb::Database;
use std::net::TcpStream;
use std::sync::Arc;

pub struct List;

impl GenericHandler for List {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        
        
        
        Ok(())
    }
}