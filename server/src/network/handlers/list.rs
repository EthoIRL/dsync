use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase, ReadableTable};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::proto::comms::ListResponse;
use crate::proto::constant::PacketKind;

pub struct List;

impl GenericHandler for List {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;
        
        println!("[*] [DSYNC] [List] Client Request");

        let mut object_ids: Vec<Vec<u8>> = Vec::new();
        let mut paths: Vec<String> = Vec::new();
        let mut is_childs: Vec<bool> = Vec::new();
        let mut parent_directorys: Vec<String> = Vec::new();

        for object_kv in object_table.iter()? {
            if let Ok(object_kv) = object_kv {
                let object_id = object_kv.0.value();
                let object: Object = bitcode::decode(&*object_kv.1.value())?;

                object_ids.push(object_id.to_vec());
                paths.push(object.path);
                is_childs.push(object.child_of_tree);
                parent_directorys.push(object.parent_tree.unwrap_or_else(String::new))
            }
        }

        let response = ListResponse {
            object_id: object_ids,
            path: paths,
            is_child: is_childs,
            parent_directory: parent_directorys
        };

        packet::send_packet(stream, &mut [PacketKind::ListResponse as u8], response)?;

        Ok(())
    }
}