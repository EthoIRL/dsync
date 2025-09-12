use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{StatusResponse, Sync, SyncResponse};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::network::tools::prototools;
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::constant::PacketKind;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync: Sync = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                // We must alert the client that the object was deleted
                let status_response = StatusResponse {
                    object_id: sync.object_id,
                    state: ObjectState::Deleted as i32
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectStatusResponse as u8], status_response)?;
            },
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                if object.is_directory {
                    return Ok(())
                }

                let Some(hashes) = object.chunk_hashes else {
                    return Err("Client requested to sync to an object we don't have chunks hashes for? (TODO: CHuNK HOW)".into())
                };

                let sync_response = SyncResponse {
                    object_id: sync.object_id,
                    object_hash: object.hash,
                    hashes
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;
            }
        }

        Ok(())
    }
}