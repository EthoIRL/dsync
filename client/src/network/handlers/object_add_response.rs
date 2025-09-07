use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use redb::{Database, ReadableDatabase};
use crate::config::Config;
use crate::network::handlers::object_status::ObjectStatus;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::AddResponse;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectAddResponse;

impl GenericHandler for ObjectAddResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let add_response: AddResponse = packet.decode()?;

        println!("Test {:#?}", add_response);
        if add_response.object_id.len() < 4 || add_response.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", add_response.object_id.len()).into())
        }

        let object_id: [u8; 4] = add_response.object_id[0..4].try_into()?;

        if !add_response.success {
            return match add_response.error {
                None => Err("AddResponse not returning a correct error message, when erroring.".into()),
                Some(err_id) => {
                    let error = AddError::try_from(err_id)?;
                    if error == AddError::AlreadySynced {
                        sync_check(object_id, add_response.path, database)?;
                    }

                    Ok(())
                }
            }
        }

        if add_response.object_id.len() > 4 {
            return Err(format!("Object ID is larger than expected? ({})", add_response.object_id.len()).into());
        }

        let write_txn = database.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
            objects.insert(object_id.clone(), add_response.path)?;
        }
        write_txn.commit()?;

        println!("Added object locally");

        Ok(())
    }
}

fn sync_check(object_id: [u8; 4], path: String, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
    println!("Sync checked again");

    let read_txn = database.begin_read()?;
    if let Ok(object_table ) = read_txn.open_table(OBJECTS_LOCAL_TABLE) {
        if object_table.get(object_id)?.is_none() {
            read_txn.close()?;

            let write_txn = database.begin_write()?;
            {
                let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
                objects.insert(object_id.clone(), path)?;
            }
            write_txn.commit()?;
        };
    }

    Ok(())
}
