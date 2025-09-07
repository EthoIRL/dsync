use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::ChunkResponse;
use crate::proto::constant::ChunkSize;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase};
use std::error::Error;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use crate::network::tools::prototools;

pub struct ObjectChunkResponse;

impl GenericHandler for ObjectChunkResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

        let object_id = prototools::get_object_id(&chunk_response.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        let path = match object_table.get(&object_id)? {
            None => return Err("Couldn't find object in local database".into()),
            Some(object_path) => {
                PathBuf::from(object_path.value())
            }
        };

        let mut file = File::open(path).expect("Failed to open file... during traversal");
        file.seek(SeekFrom::Start((chunk_response.chunk_offset * ChunkSize::Size as u32) as u64))?;
        file.write_all(&chunk_response.chunk)?;

        Ok(())
    }
}