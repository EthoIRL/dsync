use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::ChunkResponse;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

pub struct ObjectChunkResponse;

impl GenericHandler for ObjectChunkResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

        if chunk_response.object_id.len() < 4 || chunk_response.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", chunk_response.object_id.len()).into())
        }

        let object_id: [u8; 4] = chunk_response.object_id[0..4].try_into()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => return Err("No object found?".into()),
            Some(object) => {
                let mut object: Object = bitcode::decode(&*object.value())?;

                println!("Handle ChunkResponse: ({}) : {}", object.path, chunk_response.chunk_offset);

                if object.chunk_hashes.is_none() {
                    object.chunk_hashes = Some(Vec::new());
                }

                let chunk_hashes = object.chunk_hashes.as_mut().unwrap();

                let chunk_datum = chunk_response.chunk;
                let chunk_hash = xxh3_64(&chunk_datum);

                if chunk_hashes.len() <= chunk_response.chunk_offset as usize {
                    chunk_hashes.resize(chunk_response.chunk_offset as usize + 1, 0);
                }

                chunk_hashes[chunk_response.chunk_offset as usize] = chunk_hash;

                for (index,data)  in chunk_datum.iter().enumerate() {
                    if object.chunk_data.len() <= chunk_response.chunk_offset as usize + index {
                        object.chunk_data.resize(chunk_response.chunk_offset as usize + index + 1, 0);
                    }

                    object.chunk_data[chunk_response.chunk_offset as usize + index] = *data;
                }

                let write_txn = database.begin_write()?;
                {
                    let mut objects = write_txn.open_table(OBJECTS_TABLE)?;
                    objects.insert(object_id.clone(), bitcode::encode(&object))?;
                }
                write_txn.commit()?;
            }
        }


        Ok(())
    }
}