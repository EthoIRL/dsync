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