use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::{StatusResponse, Sync};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use redb::Database;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectStatusResponse;

impl GenericHandler for ObjectStatusResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let status_response: StatusResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&status_response.object_id)?;

        let state = ObjectState::try_from(status_response.state)?;

        println!("[*] [DSYNC] [StatusResponse] [{}] (State: {:#?})", prototools::object_id_hex(&object_id), state);

        match state {
            ObjectState::RemoteOutOfDate => {
                let sync_request = Sync {
                    object_id: status_response.object_id.clone(),
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
            },
            ObjectState::Deleted => {
                println!("[*] [DSYNC] Object deleted from the client [{}]", prototools::object_id_hex(&object_id));

                let write_txn = database.begin_write()?;
                let mut object_table = write_txn.open_table(OBJECTS_TABLE)?;
                object_table.remove(&object_id)?;
            }
            _ => ()
        }

        Ok(())
    }
}