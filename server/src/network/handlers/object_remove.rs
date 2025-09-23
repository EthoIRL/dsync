use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::prototools;
use crate::proto::comms::object::{Remove, RemoveResponse};
use crate::proto::constant::PacketKind;
use crate::tables::{OBJECTS_HASH_TABLE, OBJECTS_TABLE};
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;

pub struct ObjectRemove;

impl GenericHandler for ObjectRemove {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let remove_request: Remove = packet.decode()?;

        let object_id = prototools::parse_object_id(&remove_request.object_id)?;

        println!("[*] [DSYNC] [ObjectRemove] {} requested to delete object [{}]", remove_request.hostname, prototools::object_id_hex(&object_id));

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => {
                let remove_response = RemoveResponse {
                    success: false,
                    object_id: remove_request.object_id
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectRemoveResponse as u8], remove_response)?;

                return Err(format!("Couldn't find object [ID: {}]", prototools::object_id_hex(&object_id)).into());
            },
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                if object.child_of_tree {
                    // Removes object_id from parent's child_ids
                    if let Some(parent_tree) = object.parent_tree {
                        let parent_id: [u8; 4] = xxh32(format!("{}-{}", parent_tree, object.hostname).as_bytes(), 0).to_le_bytes();

                        match object_table.get(&parent_id)? {
                            None => {
                                return Err("Object is a child of a tree; couldn't find parent_id!".into());
                            },
                            Some(parent_object) => {
                                let mut parent_object: Object = bitcode::decode(&*parent_object.value())?;

                                let mut children_ids = match parent_object.children_ids {
                                    None => unreachable!("Parent has no child ids but child object requested deletion?"),
                                    Some(children_ids) => children_ids
                                };

                                let object_position = match children_ids.iter().position(|id| id == &object.object_id) {
                                    None => unreachable!("Couldn't find object's position in parent id vec!"),
                                    Some(object_position) => object_position
                                };

                                children_ids.remove(object_position);
                                parent_object.children_ids = Some(children_ids);

                                let write_txn = database.begin_write()?;
                                {
                                    let mut object_table = write_txn.open_table(OBJECTS_TABLE)?;
                                    object_table.insert(&parent_id, bitcode::encode(&parent_object))?;
                                }
                            }
                        }
                    }
                }
                prototools::delete_object(&object_id, object.chunk_count as u32, &database)?;
            }
        }

        // TODO: Maybe we can remove these assertions? for performance reasons perhaps?
        let read_txn = database.begin_read()?;
        let hash_object_table = read_txn.open_table(OBJECTS_HASH_TABLE)?;

        assert!(hash_object_table.get(&object_id)?.is_none());

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        assert!(object_table.get(&object_id)?.is_none());

        let remove_response = RemoveResponse {
            success: true,
            object_id: remove_request.object_id
        };

        println!("[*] [DSYNC] [ObjectRemove] Object successfully deleted [{}]", prototools::object_id_hex(&object_id));

        packet::send_packet(stream, &mut [PacketKind::ObjectRemoveResponse as u8], remove_response)?;

        Ok(())
    }
}