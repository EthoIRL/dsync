use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::ListResponse;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase};
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::tools::prototools;

pub struct ObjectListResponse;

impl GenericHandler for ObjectListResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let list_response: ListResponse = packet.decode()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        for ((object_id, is_child), (path, parent_directory)) in
            list_response.object_id.iter()
                .zip(list_response.is_child.iter())
                .zip(list_response.path.iter()
                .zip(list_response.parent_directory.iter()))
        {
            let object_id = prototools::get_object_id(&object_id)?;

            match object_table.get(&object_id)? {
                None => {
                    println!("[*] [DSYNC] Object removed from syncing by remote master (Path: {})", path);
                    let write_txn = database.begin_write()?;
                    {
                        let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
                        objects.remove(&object_id)?;
                    }
                    write_txn.commit()?;
                },
                Some(object) => {
                    let hex_object_id: String = object_id.iter()
                        .map(|byte| format!("{:02X}", byte))
                        .collect();

                    println!("[*] [DSYNC] [REMOTE: {}] [LOCAL: {}] [ID: {}]", path, object.value(), hex_object_id);
                }
            }
        }

        Ok(())
    }
}