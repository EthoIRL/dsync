use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use redb::{Database, ReadableDatabase};
use crate::tables::OBJECTS_LOCAL_TABLE;

pub fn parse_object_id(object_id_vec: &Vec<u8>) -> Result<[u8; 4], Box<dyn Error>> {
    if object_id_vec.len() != 4 {
        return Err(format!("Object_id is an incorrect length. (Expected: 4, Got: {})", object_id_vec.len()).into());
    }

    Ok(object_id_vec[0..4].try_into()?)
}

pub fn get_object_path(object_id: &[u8; 4], database: &Arc<Database>) -> Result<PathBuf, Box<dyn Error>> {
    let read_txn = database.begin_read()?;
    let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

    match object_table.get(object_id)? {
        None => Err("Couldn't find object in local database".into()),
        Some(object_path) => {
            Ok(PathBuf::from(object_path.value()))
        }
    }
}

pub fn object_id_hex(object_id: &[u8; 4]) -> String {
    object_id.iter()
        .map(|byte| format!("{:02X}", byte))
        .collect()
}