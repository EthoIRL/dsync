use crate::cli;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableTable, Table};
use std::error::Error;

pub fn handle_dsync(database: &Database, target: &String) -> Result<(), Box<dyn Error>>{
    let id = cli::get_id_from_target(database, target)?;
    
    let write_txn = database.begin_write()?;
    {
        let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        let object_path = match object_table.get(&id)? {
            Some(object_path) =>  object_path.value(),
            None => return Err("Couldn't find object".into())
        };

        let related_objects = dsync_find_related(&object_path, &object_table)?;
        for related_object_id in related_objects {
            object_table.remove(&related_object_id)?;
        }

        object_table.remove(&id)?;
    }
    write_txn.commit()?;

    Ok(())
}

fn dsync_find_related(parent_path: &String, table: &Table<[u8; 4], String>) -> Result<Vec<[u8; 4]>, Box<dyn Error>> {
    let mut related_object_ids: Vec<[u8; 4]> = Vec::new();

    for object in table.iter()? {
        if let Ok(object) = object {
            let object_id = object.0.value();
            let object_path = object.1.value();

            if object_path.contains(parent_path) {
                related_object_ids.push(object_id);
            }
        }
    }

    Ok(related_object_ids)
}