use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{protofile, prototools};
use crate::proto::comms::object::{Sync, SyncResponse};
use crate::proto::constant::PacketKind;
use redb::Database;
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let sync: Sync = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        if !path.exists() {
            return todo!("Reached unknown control flow point")
            // TODO: Handle auto removing
            // I think we can just ignore this, as a status request will handle this?
        }

        let object_hash = protofile::hash_object(&path)?;
        let chunk_hashes: Vec<u64> = protofile::hash_file_chunks(&path)?;

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