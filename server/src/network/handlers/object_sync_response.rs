use std::error::Error;
use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{chunktools, prototools};
use crate::proto::comms::object::{Chunk, SyncResponse};
use crate::proto::constant::PacketKind;
use crate::tables::{OBJECTS_CHUNK_TABLE, OBJECTS_TABLE};
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectSyncResponse;

impl GenericHandler for ObjectSyncResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync_response: SyncResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync_response.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                return Err("Client requested to sync to an object we don't have? (TODO: HOW)".into())
            },
            Some(object) => {
                let mut object: Object = bitcode::decode(&*object.value())?;

                match chunktools::get_hashes(&object_id, database) {
                    Err(_) => {
                        return request_all_chunks(stream, sync_response.hashes, sync_response.object_id);
                    },
                    Ok(server_hashes) => {
                        if server_hashes.is_empty() {
                            if sync_response.hashes.is_empty() {
                                if object.hash != 0 {
                                    object.hash = 0;

                                    let write_txn = database.begin_write()?;
                                    {
                                        let mut objects = write_txn.open_table(OBJECTS_TABLE)?;
                                        objects.insert(object_id.clone(), bitcode::encode(&object))?;
                                    }
                                    write_txn.commit()?;
                                }
                                
                                return Ok(())
                            }
                            return request_all_chunks(stream, sync_response.hashes, sync_response.object_id);
                        }

                        println!("[*] [DSYNC] Hash chunks {} {}", server_hashes.len(), object.path);

                        if server_hashes.len() == sync_response.hashes.len() {
                            assert_ne!(server_hashes, sync_response.hashes, "Server object hashes are the same as the sync_response, this should never happen.");
                        }

                        let chunk_diffs = diff_chunks(&sync_response.hashes, &server_hashes);

                        for (i, diff) in chunk_diffs.into_iter().enumerate() {
                            match diff {
                                ChunkDiff::ReplaceOrInsert => {
                                    println!("Insert or replace chunk at (i: {})", i);

                                    // TODO: While uploading large files that can take a few seconds we might get another sync leading to double chunk requests happening
                                    // TODO: This is inefficient, however I see no easy way to fix this while staying fault tolerant.

                                    let chunk_request = Chunk {
                                        object_id: sync_response.object_id.clone(),
                                        chunk_offset: i as u32,
                                    };

                                    packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
                                }
                                ChunkDiff::Delete => {
                                    println!("Delete chunk at index {}", i);

                                    // We already deleted them
                                    if i as u64 > object.chunk_count {
                                        return Ok(());
                                    }

                                    let mut hashes = chunktools::get_hashes(&object_id, database)?;
                                    hashes.truncate(i);
                                    chunktools::save_hashes(&object_id, hashes, database)?;

                                    let write_txn = database.begin_write()?;
                                    {
                                        let mut objects = write_txn.open_table(OBJECTS_CHUNK_TABLE)?;
                                        for chunk_index in i..(object.chunk_count as usize) {
                                            {
                                                let mut object_id_offset = [0u8; 8];
                                                object_id_offset[..4].copy_from_slice(&object_id);
                                                object_id_offset[4..].copy_from_slice(&(chunk_index as u32).to_le_bytes());

                                                objects.remove(&object_id_offset)?;
                                            }
                                        }
                                    }
                                    write_txn.commit()?;

                                    let write_txn = database.begin_write()?;
                                    object.chunk_count = i as u64;
                                    object.hash = chunktools::hash_all_chunks(&object_id, object.chunk_count as u32, database)?;
                                    {
                                        let mut objects = write_txn.open_table(OBJECTS_TABLE)?;
                                        objects.insert(object_id.clone(), bitcode::encode(&object))?;
                                    }
                                    write_txn.commit()?;
                                }
                                _ => ()
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ChunkDiff {
    Keep,
    ReplaceOrInsert,
    Delete,
}

// Client is up-to-date, while the Remote isn't.
fn diff_chunks(local: &[u64], remote: &[u64]) -> Vec<ChunkDiff> {
    let mut diffs = Vec::new();
    let max_len = local.len().max(remote.len());

    for i in 0..max_len {
        match (local.get(i), remote.get(i)) {
            (Some(&local_hash), Some(&remote_hash)) => {
                if local_hash == remote_hash {
                    diffs.push(ChunkDiff::Keep);
                } else {
                    diffs.push(ChunkDiff::ReplaceOrInsert);
                }
            }
            (None, Some(_)) => diffs.push(ChunkDiff::Delete),
            (Some(_), None) => diffs.push(ChunkDiff::ReplaceOrInsert),
            (None, None) => break,
        }
    }

    diffs
}

fn request_all_chunks(stream: &mut TcpStream, hashes: Vec<u64>, object_id: Vec<u8>) -> Result<(), Box<dyn Error>>{
    for (offset, _) in hashes.iter().enumerate() {
        let chunk_request = Chunk {
            object_id: object_id.clone(),
            chunk_offset: offset as u32
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
    }

    Ok(())
}