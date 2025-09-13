use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::ChunkResponse;
use crate::proto::constant::ChunkSize;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

pub struct ObjectChunkResponse;

const CHUNK_SIZE: usize = ChunkSize::Size as usize;

impl GenericHandler for ObjectChunkResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

        if chunk_response.chunk.len() > CHUNK_SIZE {
            return Err(format!("Chunk size too large [Expected: {}] <= [Received: {}]", CHUNK_SIZE, chunk_response.chunk.len()).into());
        }

        let object_id = prototools::parse_object_id(&chunk_response.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                return Err(format!("Object chunk received, object no longer exists.. [{}]", prototools::object_id_hex(&object_id)).into());
            },
            Some(object) => {
                let mut object: Object = bitcode::decode(&*object.value())?;
                
                if object.is_directory {
                    return Err(format!("Received data chunk for a directory? [{}]", prototools::object_id_hex(&object_id)).into());
                }

                println!("[*] [DSYNC] [ChunkResponse] {} ({})", object.path, chunk_response.chunk_offset);

                let chunk_hashes = object.chunk_hashes.get_or_insert_with(Vec::new);
                let chunk_data = &mut object.chunk_data;

                let chunk_datum = chunk_response.chunk;
                let chunk_hash = xxh3_64(&chunk_datum);
                let chunk_index_offset = chunk_response.chunk_offset as usize * CHUNK_SIZE;

                // Chunk Hashes
                if chunk_hashes.len() <= chunk_index_offset {
                    chunk_hashes.resize(chunk_index_offset + 1, 0);
                }
                chunk_hashes[chunk_index_offset] = chunk_hash;

                // Chunk Data
                if chunk_data.len() < chunk_index_offset + chunk_datum.len() {
                    chunk_data.resize(chunk_index_offset + chunk_datum.len(), 0);
                }

                for index in 0..chunk_datum.len() {
                    chunk_data[chunk_index_offset + index] = chunk_datum[index];
                }

                // Hash Object
                object.hash = xxh3_64(&chunk_data);

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