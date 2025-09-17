use crate::cli;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::Database;
use std::error::Error;

pub fn handle_dsync(database: &Database, target: &String) -> Result<(), Box<dyn Error>>{
    let id = cli::get_id_from_target(database, target)?;

    let write_txn = database.begin_write()?;
    {
        let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
        object_table.remove(&id)?;
    }
    write_txn.commit()?;

    Ok(())
}