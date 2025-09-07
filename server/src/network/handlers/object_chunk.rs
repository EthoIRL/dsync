use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{Chunk, ChunkResponse};
use crate::proto::constant::{ChunkSize, PacketKind};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::tools::prototools;

pub struct ObjectChunk;

impl GenericHandler for ObjectChunk {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_request: Chunk = packet.decode()?;

        let object_id = prototools::get_object_id(&chunk_request.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => return Err("No object found?".into()),
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                println!("Handle Chunk Request: ({}) : {}", object.path, chunk_request.chunk_offset);

                let request_data = object.chunk_data[(chunk_request.chunk_offset * ChunkSize::Size as u32) as usize..ChunkSize::Size as usize].to_vec();

                let chunk_response = ChunkResponse {
                    object_id: chunk_request.object_id,
                    chunk_offset: chunk_request.chunk_offset,
                    chunk: request_data
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectChunkResponse as u8], chunk_response)?;
            }
        }


        Ok(())
    }
}