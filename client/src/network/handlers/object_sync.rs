use std::error::Error;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use redb::{Database, ReadableDatabase};
use crate::config::Config;
use crate::network::handlers::object_status::hash_object;
use crate::network::handlers::object_sync_response::hash_file_chunks;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{Sync, SyncResponse};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let sync: Sync = packet.decode()?;

        if sync.object_id.len() < 4 || sync.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", sync.object_id.len()).into())
        }

        let object_id: [u8; 4] = sync.object_id[0..4].try_into()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        let path = match object_table.get(&object_id)? {
            None => return Err("Couldn't find object in local database".into()),
            Some(object_path) => {
                PathBuf::from(object_path.value())
            }
        };

        if !path.exists() {
            // TODO: Handle auto removing
            return Err(format!("Object {:?} does not exist!", path).into())
        }

        let object_hash = hash_object(&path)?;
        let chunk_hashes: Vec<u64> = hash_file_chunks(&path)?;

        // TODO: To handle sync response more appropriately, there should be a enum of the file state.
        // E.g. Fine, Deleted

        let sync_response = SyncResponse {
            object_id: sync.object_id,
            object_hash,
            hashes: chunk_hashes
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;

        Ok(())
    }
}