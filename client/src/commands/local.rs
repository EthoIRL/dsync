use crate::cli;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableTable};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn handle_dsync(database: &Database, target: &String) -> Result<(), Box<dyn Error>>{
    let id = cli::get_id_from_target(database, target)?;
    
    let write_txn = database.begin_write()?;
    {
        let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        let object_path = match object_table.get(&id)? {
            Some(object_path) =>  object_path.value(),
            None => return Err("Couldn't find object".into())
        };

        let object_path = PathBuf::from(object_path);

        if object_path.is_dir() {
            let paths_to_remove = dsync_recursion_traversal(&object_path);

            if !paths_to_remove.is_empty() {
                let mut ids: Vec<[u8; 4]> = Vec::new();

                for object in object_table.iter()? {
                    let object = object?;

                    let object_id = object.0.value();
                    let object_path = object.1.value();

                    if paths_to_remove.contains(&object_path) {
                        ids.push(object_id);
                    }
                }

                for id in ids {
                    object_table.remove(&id)?;
                }
            }
        }

        object_table.remove(&id)?;
    }
    write_txn.commit()?;

    Ok(())
}


fn dsync_recursion_traversal(directory: &PathBuf) -> Vec<String> {
    let mut paths_to_remove: Vec<String> = Vec::new();

    fs::read_dir(&directory).unwrap()
        .for_each(|entry| {
            if let Ok(entry) = entry {
                println!("Entry: {}", entry.path().display());
                if entry.path().is_dir() {
                    paths_to_remove.append(&mut dsync_recursion_traversal(&entry.path()));
                }

                paths_to_remove.push(entry.path().to_str().unwrap().to_string());
            }
        });

    paths_to_remove
}