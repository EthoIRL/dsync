use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::ChunkResponse;
use crate::proto::constant::ChunkSize;
use redb::Database;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectChunkResponse;

impl GenericHandler for ObjectChunkResponse {
    fn handle(_: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&chunk_response.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        file.seek(SeekFrom::Start((chunk_response.chunk_offset * ChunkSize::Size as u32) as u64))?;
        file.write_all(&chunk_response.chunk)?;

        let last_modification = file.metadata()?.modified()?;
        if chunk_response.chunk.len() < ChunkSize::Size as usize {
            let total_file_size = (chunk_response.chunk_offset as u64 * ChunkSize::Size as u64) + chunk_response.chunk.len() as u64;
            file.set_len(total_file_size)?;
        }
        file.set_modified(last_modification)?;
        file.flush()?;

        Ok(())
    }
}