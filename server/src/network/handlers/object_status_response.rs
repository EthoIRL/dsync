use crate::config::Config;
use crate::network::handlers::object_sync::ObjectSync;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::StatusResponse;
use redb::Database;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectStatusResponse;

impl GenericHandler for ObjectStatusResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let status_response: StatusResponse = packet.decode()?;
        
        println!("Handling object status response");

        let state = ObjectState::try_from(status_response.state)?;

        if state == ObjectState::Fine {
            return Ok(())
        }

        match state {
            ObjectState::Fine => {
                return Ok(())
            },
            ObjectState::RemoteOutOfDate => {
                
                
                todo!()
            },
            ObjectState::LocalOutOfDate => {
                todo!()
            }
        }
    }
}