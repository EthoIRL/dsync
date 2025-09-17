use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::ListResponse;
use crate::tables::OBJECTS_LOCAL_TABLE;
use redb::{Database, ReadableDatabase};
use std::error::Error;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use crate::network::tools::prototools;

pub struct ObjectListResponse;

impl GenericHandler for ObjectListResponse {
    fn handle(_: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let list_response: ListResponse = packet.decode()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_LOCAL_TABLE)?;

        for ((object_id, is_child), (path, parent_directory)) in
            list_response.object_id.iter()
                .zip(list_response.is_child.iter())
                .zip(list_response.path.iter()
                .zip(list_response.parent_directory.iter()))
        {
            if *is_child {
                continue;
            }

            let object_id = prototools::parse_object_id(&object_id)?;

            match object_table.get(&object_id)? {
                None => {
                    println!("[*] [DSYNC] Not Synced: [{}] [{}]", path, prototools::object_id_hex(&object_id));
                },
                Some(object) => {
                    if let Some(file_name) = PathBuf::from(path).file_name() {
                        if let Some(file_name) = file_name.to_str() {
                            println!("[*] [DSYNC] [ListResponse] [{}]<->[{}] [ID: {}]", file_name, object.value(), prototools::object_id_hex(&object_id));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}