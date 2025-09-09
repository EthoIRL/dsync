use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::{StatusResponse, Sync};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::tools::prototools;

pub struct ObjectStatusResponse;

impl GenericHandler for ObjectStatusResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let status_response: StatusResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&status_response.object_id)?;

        println!("Handling object status response");

        let state = ObjectState::try_from(status_response.state)?;

        match state {
            ObjectState::Fine => {
                todo!()
            },
            ObjectState::RemoteOutOfDate => {
                let sync_request = Sync {
                    object_id: status_response.object_id.clone(),
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
            },
            ObjectState::LocalOutOfDate => {
                todo!()
            }
            ObjectState::Deleted => {
                // TODO: Fix this with prototools hex to string
                println!("[*] [DSYNC] Object deleted from the client");

                let write_txn = database.begin_write()?;
                let mut object_table = write_txn.open_table(OBJECTS_TABLE)?;
                object_table.remove(&object_id)?;
            }
        }

        Ok(())
    }
}