use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::ChunkResponse;
use crate::proto::constant::ChunkSize;
use redb::Database;
use std::error::Error;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectChunkResponse;

impl GenericHandler for ObjectChunkResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&chunk_response.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        let mut file = File::open(path).expect("Failed to open file... during traversal");
        file.seek(SeekFrom::Start((chunk_response.chunk_offset * ChunkSize::Size as u32) as u64))?;
        file.write_all(&chunk_response.chunk)?;

        Ok(())
    }
}