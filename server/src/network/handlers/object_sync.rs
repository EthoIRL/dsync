use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{Sync, SyncResponse};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::network::tools::prototools;
use crate::proto::constant::PacketKind;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync: Sync = packet.decode()?;

        let object_id = prototools::get_object_id(&sync.object_id)?;

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
                        return Err("Client requested to sync to an object we don't have chunks hashes for? (TODO: CHuNK HOW)".into())
                    },
                    Some(hashes ) => {
                        let sync_response = SyncResponse {
                            object_id: sync.object_id,
                            object_hash: object.hash,
                            hashes
                        };

                        packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;
                    }
                }
            }
        }

        Ok(())
    }
}