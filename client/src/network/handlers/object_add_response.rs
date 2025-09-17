use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::AddResponse;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::Database;
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectAddResponse;

impl GenericHandler for ObjectAddResponse {
    fn handle(_: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let add_response: AddResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&add_response.object_id)?;

        println!("[*] [DSYNC] [AddResponse] {:#?} [{}] [Success: {}]", add_response.path, prototools::object_id_hex(&object_id), add_response.success);

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

        let write_txn = database.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
            objects.insert(object_id.clone(), add_response.path)?;
        }
        write_txn.commit()?;

        Ok(())
    }
}

fn sync_check(object_id: [u8; 4], path: String, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
    if prototools::get_object_path(&object_id, &database).is_err() {
        let write_txn = database.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
            objects.insert(object_id.clone(), path)?;
        }
        write_txn.commit()?;
    };

    Ok(())
}
