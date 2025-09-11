use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::{Chunk, ChunkResponse, Sync};
use crate::proto::constant::{ChunkSize, PacketKind};
use redb::Database;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectChunk;

impl GenericHandler for ObjectChunk {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let chunk_request: Chunk = packet.decode()?;

        let object_id = prototools::parse_object_id(&chunk_request.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        if !path.exists() {
            // Force a sync. Client is out of date?
            // If the server is requesting a chunk, than we must assume the server has a copy or it's out of date.
            
            let sync_request = Sync {
                object_id: chunk_request.object_id
            };
            
            packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
            
            return Ok(());
        }

        println!("[*] [DSYNC] [ChunkRequest] {:?} ({})", path.clone(), chunk_request.chunk_offset);

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