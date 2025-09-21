use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::{Children, ChildrenResponse, ObjectType};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectChildren;

impl GenericHandler for ObjectChildren {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let children_request: Children = packet.decode()?;

        let object_id = prototools::parse_object_id(&children_request.object_id)?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                return Err(format!("Client requested object that doesn't exist [ID: {}]", prototools::object_id_hex(&object_id)).into());
            },
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                println!("[*] [DSYNC] [ObjectChildren] Client requested all children_ids [{}]", prototools::object_id_hex(&object_id));
                let (children_ids, (children_names, children_types)): (Vec<Vec<u8>>, (Vec<String>, Vec<i32>)) = match &object.children_ids {
                    None => return Ok(()),
                    Some(children_ids) => {
                        children_ids
                            .iter()
                            .filter_map(|id| {
                                let child_id_vec = id.to_vec();

                                match object_table.get(id) {
                                    Err(_) => return None,
                                    Ok(encoded_object) => match encoded_object {
                                        None => return None,
                                        Some(encoded_object) => {
                                            let encoded_object = encoded_object;
                                            let child: Object = bitcode::decode(&*encoded_object.value()).unwrap();

                                            let name = child.path.replace(&format!("{}/", &object.path), "");
                                            let obj_type = if child.is_directory {
                                                ObjectType::Directory as i32
                                            } else {
                                                ObjectType::File as i32
                                            };

                                            Some((child_id_vec, (name, obj_type)))
                                        }
                                    }
                                }
                            })
                            .unzip()
                    }
                };

                let children_response = ChildrenResponse {
                    parent_id: children_request.object_id,
                    children_names,
                    children_types,
                    children_ids
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectChildrenResponse as u8], children_response)?;
            }
        }

        Ok(())
    }
}