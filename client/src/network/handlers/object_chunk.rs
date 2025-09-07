use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{Chunk, ChunkResponse};
use crate::proto::constant::{ChunkSize, PacketKind};
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ObjectChunk;

impl GenericHandler for ObjectChunk {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let chunk_request: Chunk = packet.decode()?;

        if chunk_request.object_id.len() < 4 || chunk_request.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", chunk_request.object_id.len()).into())
        }

        let object_id: [u8; 4] = chunk_request.object_id[0..4].try_into()?;

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

        println!("Handling Object chunk request: ({:?}) : {}", path.clone(), chunk_request.chunk_offset);

        let mut file = File::open(path).expect("Failed to open file... during traversal");

        let mut data = vec![0u8; ChunkSize::Size as usize];
        file.seek(SeekFrom::Start((chunk_request.chunk_offset * ChunkSize::Size as u32) as u64))?;
        let bytes_read = file.read(&mut data)?;
        data.truncate(bytes_read);

        let chunk_response = ChunkResponse {
            object_id: chunk_request.object_id,
            chunk_offset: chunk_request.chunk_offset,
            chunk: data
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectChunkResponse as u8], chunk_response)?;

        Ok(())
    }
}