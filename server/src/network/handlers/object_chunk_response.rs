use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::ChunkResponse;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;
use crate::network::tools::prototools;

pub struct ObjectChunkResponse;

impl GenericHandler for ObjectChunkResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let chunk_response: ChunkResponse = packet.decode()?;

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

                object.hash = xxh3_64(&object.chunk_data);

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