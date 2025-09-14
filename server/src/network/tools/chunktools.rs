use crate::tables::{OBJECTS_CHUNK_TABLE, OBJECTS_HASH_TABLE};
use redb::{Database, ReadableDatabase};
use std::error::Error;
use xxhash_rust::xxh3::xxh3_64;
use crate::proto::constant::ChunkSize;

pub fn get_chunk(object_id: &[u8; 4], offset: u32, database: &Database) -> Result<Vec<u8>, Box<dyn Error>>{
    let mut object_id_offset = [0u8; 8];
    object_id_offset[..4].copy_from_slice(object_id);
    object_id_offset[4..].copy_from_slice(&offset.to_le_bytes());

    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_CHUNK_TABLE)?;

    match object_table.get(&object_id_offset)? {
        None => Err("Couldn't find chunk".into()),
        Some(object) => {
            Ok(object.value())
        }
    }
}

pub fn save_chunk(object_id: &[u8; 4], offset: u32, chunk_data: Vec<u8>, database: &Database) -> Result<(), Box<dyn Error>>{
    let mut object_id_offset = [0u8; 8];
    object_id_offset[..4].copy_from_slice(object_id);
    object_id_offset[4..].copy_from_slice(&offset.to_le_bytes());

    let write_txn = database.begin_write()?;
    {
        let mut objects = write_txn.open_table(OBJECTS_CHUNK_TABLE)?;
        objects.insert(object_id_offset, chunk_data)?;
    }
    write_txn.commit()?;

    Ok(())
}

pub fn all_chunks_present(object_id: &[u8; 4], offsets_max: u32, database: &Database) -> Result<bool, Box<dyn Error>> {
    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_CHUNK_TABLE)?;

    for offset in 0..=offsets_max {
        let mut object_id_offset = [0u8; 8];
        object_id_offset[..4].copy_from_slice(object_id);
        object_id_offset[4..].copy_from_slice(&offset.to_le_bytes());

        if object_table.get(&object_id_offset)?.is_none() {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn hash_all_chunks(object_id: &[u8; 4], offsets_max: u32, database: &Database) -> Result<u64, Box<dyn Error>>{
    let mut data: Vec<u8> = Vec::with_capacity(offsets_max as usize * ChunkSize::Size as usize);

    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_CHUNK_TABLE)?;

    for offset in 0..=offsets_max {
        let mut object_id_offset = [0u8; 8];
        object_id_offset[..4].copy_from_slice(object_id);
        object_id_offset[4..].copy_from_slice(&offset.to_le_bytes());

        match object_table.get(&object_id_offset)? {
            None => return Err("Couldn't find chunk".into()),
            Some(object) => {
                data.append(&mut object.value());
            }
        }
    }

    Ok(xxh3_64(&data))
}

pub fn get_hashes(object_id: &[u8; 4], database: &Database) -> Result<Vec<u64>, Box<dyn Error>>{
    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_HASH_TABLE)?;

    match object_table.get(object_id)? {
        None => Err("Couldn't find hash".into()),
        Some(object) => {
            Ok(object.value())
        }
    }
}

pub fn save_hashes(object_id: &[u8; 4], hashes: Vec<u64>, database: &Database) -> Result<(), Box<dyn Error>>{
    let write_txn = database.begin_write()?;
    {
        let mut objects = write_txn.open_table(OBJECTS_HASH_TABLE)?;
        objects.insert(object_id.clone(), hashes)?;
    }
    write_txn.commit()?;

    Ok(())
}