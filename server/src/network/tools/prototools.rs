use std::error::Error;
use redb::Database;
use crate::network::handlers::object_add::Object;
use crate::tables::{OBJECTS_CHUNK_TABLE, OBJECTS_HASH_TABLE, OBJECTS_TABLE};

pub fn parse_object_id(object_id_vec: &Vec<u8>) -> Result<[u8; 4], Box<dyn Error>> {
    if object_id_vec.len() != 4 {
        return Err(format!("Object_id is an incorrect length. (Expected: 4, Got: {})", object_id_vec.len()).into());
    }

    Ok(object_id_vec[0..4].try_into()?)
}

pub fn object_id_hex(object_id: &[u8; 4]) -> String {
    object_id.iter()
        .map(|byte| format!("{:02X}", byte))
        .collect()
}

pub fn delete_object(object_id: &[u8; 4], chunk_count: u32, database: &Database) -> Result<(), Box<dyn Error>> {
    let chunk_write_txn = database.begin_write()?;
    {
        let mut chunk_table = chunk_write_txn.open_table(OBJECTS_CHUNK_TABLE)?;

        for offset in 0..chunk_count {
            let mut object_id_offset = [0u8; 8];
            object_id_offset[..4].copy_from_slice(object_id);
            object_id_offset[4..].copy_from_slice(&offset.to_le_bytes());

            chunk_table.remove(&object_id_offset)?;
        }
    }
    chunk_write_txn.commit()?;

    let hash_write_txn = database.begin_write()?;
    {
        let mut hash_table = hash_write_txn.open_table(OBJECTS_HASH_TABLE)?;

        hash_table.remove(object_id)?;
    }
    hash_write_txn.commit()?;

    let object_write_txn = database.begin_write()?;
    {
        let mut object_table = object_write_txn.open_table(OBJECTS_TABLE)?;

        object_table.remove(object_id)?;
    }
    object_write_txn.commit()?;

    Ok(())
}