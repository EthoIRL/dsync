use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::{StatusResponse, Sync};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
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

                let read_txn = database.begin_read()?;
                let object_table = read_txn.open_table(OBJECTS_TABLE)?;

                if let Some(object) = object_table.get(&object_id)? {
                    let object: Object = bitcode::decode(&*object.value())?;

                    prototools::delete_object(&object_id, object.chunk_count as u32, &database)?;
                }
            }
            _ => ()
        }

        Ok(())
    }
}