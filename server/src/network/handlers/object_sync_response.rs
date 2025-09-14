use std::hash::Hash;
use std::net::TcpStream;
use std::sync::Arc;
use redb::{Database, ReadableDatabase};
use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{chunktools, prototools};
use crate::proto::comms::object::{Chunk, SyncResponse};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;

pub struct ObjectSyncResponse;

impl GenericHandler for ObjectSyncResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let sync_response: SyncResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync_response.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                return Err("Client requested to sync to an object we don't have? (TODO: HOW)".into())
            },
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                match chunktools::get_hashes(&object_id, database) {
                    Err(_) => {
                        for (offset, _) in sync_response.hashes.iter().enumerate() {
                            let chunk_request = Chunk {
                                object_id: sync_response.object_id.clone(),
                                chunk_offset: offset as u32
                            };

                            packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
                        }
                    },
                    Ok(server_hashes) => {
                        if server_hashes.is_empty() {
                            for (offset, _) in sync_response.hashes.iter().enumerate() {
                                let chunk_request = Chunk {
                                    object_id: sync_response.object_id.clone(),
                                    chunk_offset: offset as u32
                                };

                                packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
                            }

                            return Ok(());
                        }

                        println!("[*] [DSYNC] Hash chunks {} {}", server_hashes.len(), object.path);

                        if server_hashes.len() == sync_response.hashes.len() {
                            assert_ne!(server_hashes, sync_response.hashes, "Server object hashes are the same as the sync_response, this should never happen.");
                        }

                        let chunk_diffs = diff_chunks(&sync_response.hashes, &server_hashes);

                        for (i, diff) in chunk_diffs.into_iter().enumerate() {
                            match diff {
                                ChunkDiff::Keep(idx) => {
                                    println!("Keep (IDX: {}, i: {})", idx, i);
                                    // No action needed, chunk is the same
                                }
                                ChunkDiff::Replace(idx) => {
                                    println!("Replace chunk at (IDX: {}, i: {})", idx, i);
                                    // Remove and request new chunk from remote
                                }
                                ChunkDiff::Insert => {
                                    println!("Insert new chunk at (i: {})", i);
                                    println!("Insert new chunk");
                                    // Append new chunk at the end or insert at position if needed
                                }
                                ChunkDiff::Delete(idx) => {
                                    println!("Delete chunk at index {}", idx);
                                    // Remove this chunk from the file
                                }
                            }
                        }


                        todo!()
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ChunkDiff {
    Keep(usize),          // Index in local file that matches remote
    Replace(usize),       // Index in local file needs to be replaced
    Insert,               // Chunk exists in remote but not in local
    Delete(usize),        // Chunk exists in local but not in remote
}

fn diff_chunks(local: &[u64], remote: &[u64]) -> Vec<ChunkDiff> {
    let mut diffs = Vec::new();
    let max_len = local.len().max(remote.len());

    for i in 0..max_len {
        match (local.get(i), remote.get(i)) {
            (Some(&local_hash), Some(&remote_hash)) => {
                if local_hash == remote_hash {
                    diffs.push(ChunkDiff::Keep(i));
                } else {
                    diffs.push(ChunkDiff::Replace(i));
                }
            }
            (None, Some(_)) => {
                diffs.push(ChunkDiff::Insert); // New chunk added
            }
            (Some(_), None) => {
                diffs.push(ChunkDiff::Delete(i)); // Chunk deleted
            }
            (None, None) => break,
        }
    }

    diffs
}