use std::error::Error;
use std::fs;
use std::fs::File;
use std::net::TcpStream;
use std::sync::Arc;
use redb::Database;
use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::{ChildrenResponse, ObjectType, Status};
use crate::proto::constant::{PacketKind};
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectChildrenResponse;

impl GenericHandler for ObjectChildrenResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let children_response: ChildrenResponse = packet.decode()?;

        let parent_id = prototools::parse_object_id(&children_response.parent_id)?;

        // We can assume the parent_path is a directory
        let parent_path = prototools::get_object_path(&parent_id, &database)?;

        if !parent_path.exists() {
            fs::create_dir_all(&parent_path)?;
        }

        for index in 0..children_response.children_ids.len() {
            let child_id = prototools::parse_object_id(&children_response.children_ids[index])?;
            let child_name = &children_response.children_names[index];
            let child_type = ObjectType::try_from(children_response.children_types[index])?;
            let child_path = parent_path.join(child_name);

            let child_path_string = match child_path.to_str() {
                None => continue,
                Some(child_path_str) => child_path_str.to_string()
            };

            let write_txn = database.begin_write()?;
            {
                let mut objects = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
                objects.insert(child_id, child_path_string)?;
            }
            write_txn.commit()?;

            if !child_path.exists() {
                if child_type == ObjectType::Directory {
                    fs::create_dir_all(&child_path)?;
                } else if child_type == ObjectType::File {
                    if let Some(parent) = &child_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    File::create(&child_path)?;
                }
            }
        }


        for children_id in children_response.children_ids {
            let status_request = Status {
                object_id: children_id,
                hash: None,
                modified_last: None
            };

            packet::send_packet(stream, &mut [PacketKind::ObjectStatus as u8], status_request)?;
        }

        Ok(())
    }
}