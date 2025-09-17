use crate::cli;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase, ReadableTable};
use std::error::Error;
use std::path::PathBuf;

pub fn handle_dsync(database: &Database, target: &String) -> Result<(), Box<dyn Error>>{
    let id = match cli::hex_id_to_u8_array(&target) {
        Ok(id) => id,
        Err(_) => {
            // Assume hex isn't an ID
            let potential_path = PathBuf::from(&target);

            if !potential_path.exists() {
                return Err("Path or ID is correct".into());
            }

            let read_txn = database.begin_read()?;
            let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

            let potential_object = object_table.iter()?.find(|result| {
                if let Ok((_, value)) = result {
                    if value.value().eq_ignore_ascii_case(&target) {
                        return true;
                    }
                }

                false
            });

            let object = match potential_object {
                Some(object) => object,
                None => return Err("Couldn't find a path".into())
            }?;

            object.0.value()
        }
    };

    let write_txn = database.begin_write()?;
    {
        let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
        object_table.remove(&id)?;
    }
    write_txn.commit()?;

    Ok(())
}