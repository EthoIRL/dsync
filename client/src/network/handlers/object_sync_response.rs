use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::SyncResponse;
use crate::proto::constant::ChunkSize;
use redb::Database;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

pub struct ObjectSyncResponse;

impl GenericHandler for ObjectSyncResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let sync_response: SyncResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync_response.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        if !path.exists() {
            // TODO: Handle auto removing
            return Err(format!("Object {:?} does not exist!", path).into())
        }


        // The client has requested this data, so we can assume the clients object is out-of-date.
        let local_hashes: Vec<u64> = hash_file_chunks(&path)?;
        let remote_hashes: Vec<u64> = sync_response.hashes;


        let chunk_diffs = diff_chunks(&local_hashes, &remote_hashes);

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

        Ok(())
    }
}

pub fn hash_file_chunks(object_path: &PathBuf) -> Result<Vec<u64>, Box<dyn Error>> {
    if object_path.is_dir() {
        return Err("Cannot hash directory into chunks".into());
    }

    let mut file = File::open(object_path.clone()).expect("Failed to open file... during traversal");
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data)?;

    let mut chunk_hashes: Vec<u64> = Vec::new();
    for chunk in data.chunks(ChunkSize::Size as usize) {
        chunk_hashes.push(xxh3_64(chunk));
    };

    println!("Hash chunks: {:#?} {:?}", object_path.clone(), chunk_hashes.len());

    Ok(chunk_hashes)
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