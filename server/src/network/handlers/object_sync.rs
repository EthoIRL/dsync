use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{StatusResponse, Sync, SyncResponse};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::network::tools::{chunktools, prototools};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::sync_response::SyncType;
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

                let sync_response = match object.is_directory {
                    true => {
                        SyncResponse {
                            object_id: sync.object_id,
                            object_hash: object.hash,
                            hashes: Vec::new(),
                            r#type: SyncType::Directory as i32,
                        }
                    },
                    false => {
                        let hashes = chunktools::get_hashes(&object_id, database)?;

                        SyncResponse {
                            object_id: sync.object_id,
                            object_hash: object.hash,
                            hashes,
                            r#type: SyncType::File as i32
                        }
                    }
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;

                if object.is_directory {
                    // TODO: We should send all children related to the directory, otherwise we don't even know whats inside
                    // TODO: Maybe we can change the SyncResponse protocol to indicate whether its a dir
                }
            }
        }

        Ok(())
    }
}