use std::hash::Hash;
use std::net::TcpStream;
use std::sync::Arc;
use redb::{Database, ReadableDatabase};
use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::{Chunk, SyncResponse};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;

pub struct ObjectSyncResponse;

impl GenericHandler for ObjectSyncResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync_response: SyncResponse = packet.decode()?;

        let object_id = prototools::get_object_id(&sync_response.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                return Err("Client requested to sync to an object we don't have? (TODO: HOW)".into())
            },
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                match object.chunk_hashes {
                    None => {
                        for (offset, _) in sync_response.hashes.iter().enumerate() {
                            let chunk_request = Chunk {
                                object_id: sync_response.object_id.clone(),
                                chunk_offset: offset as u32
                            };

                            packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
                        }
                    },
                    Some(chunk_hashes) => {
                        todo!()
                    }
                }
            }
        }

        Ok(())
    }
}