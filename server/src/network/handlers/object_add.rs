use std::net::TcpStream;
use crate::network::packet::{GenericHandler, GenericPacket};

pub struct ObjectAdd;

impl GenericHandler for ObjectAdd {
    fn handle(stream: &mut TcpStream, packet: GenericPacket) -> Result<(), Box<dyn std::error::Error>> {
        
        todo!()
    }
}