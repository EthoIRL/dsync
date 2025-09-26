use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{protofile, prototools};
use crate::proto::comms::object::{ObjectType, StatusResponse, Sync, SyncResponse};
use crate::proto::constant::PacketKind;
use redb::Database;
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use crate::proto::comms::object::status_response::ObjectState;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectSync;

impl GenericHandler for ObjectSync {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let sync: Sync = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        if config.debug {
            println!("[*] [DSYNC] [Sync] Server requested sync [{}] [{:#?}]", prototools::object_id_hex(&object_id), path.display());
        }

        if !path.exists() {
            println!("[*] [DSYNC] [Sync] Object no longer exists; deleting. [{}]", prototools::object_id_hex(&object_id));
            let status_response = StatusResponse {
                object_id: sync.object_id,
                state: ObjectState::Deleted as i32,
                tree_start: false,
            };

            packet::send_packet(stream, &mut [PacketKind::ObjectStatusResponse as u8], status_response)?;

            let write_txn = database.begin_write()?;
            {
                let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
                object_table.remove(&object_id)?;
            }
            write_txn.commit()?;

            return Ok(());
        }

        let object_hash = protofile::hash_object(&path)?;
        let chunk_hashes: Vec<u64> = match path.is_dir() {
            false => protofile::hash_file_chunks(&path)?,
            true => Vec::new()
        };

        let sync_response = SyncResponse {
            object_id: sync.object_id,
            object_hash,
            hashes: chunk_hashes,
            r#type: match path.is_dir() {
                true => ObjectType::Directory as i32,
                false => ObjectType::File as i32
            }
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectSyncResponse as u8], sync_response)?;

        Ok(())
    }
}