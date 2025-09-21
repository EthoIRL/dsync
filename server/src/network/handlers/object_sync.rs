use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{ObjectType, StatusResponse, Sync, SyncResponse};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::network::tools::{chunktools, prototools};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::constant::PacketKind;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync: Sync = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                // We must alert the client that the object was deleted
                let status_response = StatusResponse {
                    object_id: sync.object_id,
                    state: ObjectState::Deleted as i32,
                    tree_start: false
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
                            r#type: ObjectType::Directory as i32,
                        }
                    },
                    false => {
                        let hashes = chunktools::get_hashes(&object_id, database)?;

                        SyncResponse {
                            object_id: sync.object_id,
                            object_hash: object.hash,
                            hashes,
                            r#type: ObjectType::File as i32
                        }
                    }
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;

                if object.is_directory {
                    if let Some(children_ids) = object.children_ids {
                        for children_id in children_ids {
                            let read_txn = database.begin_read()?;
                            let object_table = read_txn.open_table(OBJECTS_TABLE)?;

                            if let Some(child_object) = object_table.get(&children_id)? {
                                let child_object: Object = bitcode::decode(&*child_object.value())?;

                                let child_sync_response = SyncResponse {
                                    object_id: children_id.to_vec(),
                                    object_hash: child_object.hash,
                                    hashes: chunktools::get_hashes(&children_id, &database)?,
                                    r#type: match child_object.is_directory {
                                        true => ObjectType::Directory as i32,
                                        false => ObjectType::File as i32
                                    }
                                };

                                packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], child_sync_response)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}