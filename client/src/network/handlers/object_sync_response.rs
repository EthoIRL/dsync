use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{protofile, prototools};
use crate::proto::comms::object::{Chunk, SyncResponse};
use redb::Database;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::proto::constant::{ChunkSize, PacketKind};

pub struct ObjectSyncResponse;

impl GenericHandler for ObjectSyncResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let sync_response: SyncResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&sync_response.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        // TODO: This goes hand in hand with Server/object_sync.rs, we need to know when its a dir

        if !path.exists() {
            File::create(&path)?;
        }

        // The client has requested this data, so we can assume the clients object is out-of-date.
        let local_hashes: Vec<u64> = protofile::hash_file_chunks(&path)?;
        let remote_hashes: Vec<u64> = sync_response.hashes;

        println!("[*] [DSYNC] Hash chunks: {:#?} {:?}", &path, local_hashes.len());

        let chunk_diffs = diff_chunks(&local_hashes, &remote_hashes);

        for (i, diff) in chunk_diffs.into_iter().enumerate() {
            match diff {
                ChunkDiff::ReplaceOrInsert => {
                    println!("Insert or replace chunk at (i: {})", i);

                    let chunk_request = Chunk {
                        object_id: sync_response.object_id.clone(),
                        chunk_offset: i as u32,
                    };

                    packet::send_packet(stream, &mut [PacketKind::ObjectChunk as u8], chunk_request)?;
                }
                ChunkDiff::Delete => {
                    println!("Delete chunk at index {}", i);

                    let mut file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&path)?;

                    let delete_up_to = i as u64 * ChunkSize::Size as u64;

                    if file.metadata()?.len() < delete_up_to {
                        return Ok(());
                    }

                    file.set_len(delete_up_to)?;
                    file.flush()?;
                }
                _ => ()
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

// Remote is up-to-date, while the Client isn't.
fn diff_chunks(local: &[u64], remote: &[u64]) -> Vec<ChunkDiff> {
    let mut diffs = Vec::new();
    let max_len = remote.len().max(local.len());

    for i in 0..max_len {
        match (local.get(i), remote.get(i)) {
            (Some(&local_hash), Some(&remote_hash)) => {
                if local_hash == remote_hash {
                    diffs.push(ChunkDiff::Keep);
                } else {
                    diffs.push(ChunkDiff::ReplaceOrInsert);
                }
            }
            (None, Some(_)) => diffs.push(ChunkDiff::ReplaceOrInsert),
            (Some(_), None) => diffs.push(ChunkDiff::Delete),
            (None, None) => break,
        }
    }

    diffs
}