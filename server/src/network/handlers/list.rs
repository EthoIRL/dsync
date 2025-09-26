use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase, ReadableTable};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use crate::network::packet;
use crate::proto::comms::ListResponse;
use crate::proto::constant::PacketKind;

pub struct List;

impl GenericHandler for List {
    fn handle(stream: &mut TcpStream, _: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;
        
        println!("[*] [DSYNC] [List] Client requested all objects");

        let mut object_ids: Vec<Vec<u8>> = Vec::new();
        let mut is_directories: Vec<bool> = Vec::new();
        let mut is_childs: Vec<bool> = Vec::new();
        let mut names: Vec<String> = Vec::new();

        for object_kv in object_table.iter()? {
            if let Ok(object_kv) = object_kv {
                let object_id = object_kv.0.value();
                let object: Object = bitcode::decode(&*object_kv.1.value())?;

                println!("{} {} {} {:#?} {} {:#?}", object.path, object.is_directory, object.hostname, object.parent_tree, object.child_of_tree, object.children_ids);

                object_ids.push(object_id.to_vec());
                is_directories.push(object.is_directory);

                let name = match object.parent_tree {
                    None => {
                        PathBuf::from(object.path).file_name().unwrap().to_str().unwrap().to_string()
                    },
                    Some(parent_tree) => {
                        let object_path = PathBuf::from(&object.path);
                        let parent_path = PathBuf::from(parent_tree);

                        match object_path.strip_prefix(parent_path) {
                            Err(_) => continue,
                            Ok(name_path) => match name_path.to_str() {
                                None => continue,
                                Some(name_path) => name_path.to_string(),
                            }
                        }
                    }
                };
                names.push(name);
                is_childs.push(object.child_of_tree);
            }
        }

        let response = ListResponse {
            object_id: object_ids,
            is_directory: is_directories,
            is_child: is_childs,
            name: names,
        };

        packet::send_packet(stream, &mut [PacketKind::ListResponse as u8], response)?;

        Ok(())
    }
}