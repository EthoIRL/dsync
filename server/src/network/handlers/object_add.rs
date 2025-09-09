use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::{Add, AddResponse, Status, Sync};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use bitcode::{Decode, Encode};
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use xxhash_rust::xxh32::xxh32;

#[derive(Debug, Decode, Encode, PartialEq)]
pub struct Object {
    pub hostname: String,

    pub parent_tree: Option<String>,
    pub child_of_tree: bool,

    pub path: String,
    pub is_directory: bool,

    pub hash: u64,
    pub last_modified: u64,
    pub object_id: [u8; 4],

    pub chunk_hashes: Option<Vec<u64>>,
    pub chunk_data: Vec<u8>,
}

impl GenericHandler for Object {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let add_object: Add = packet.decode()?;

        if add_object.path.is_empty() {
            return Err("Path of add_object is empty".into());
        }

        if add_object.hostname.is_empty() {
            return Err("Hostname of add_object is empty".into());
        }

        let object_id: [u8; 4] = xxh32(format!("{}-{}", add_object.path, add_object.hostname).as_bytes(), 0).to_le_bytes();

        let read_txn = database.begin_read()?;
        if let Ok(object_table ) = read_txn.open_table(OBJECTS_TABLE) {
            if object_table.get(&object_id)?.is_some() {
                println!("[*] [DSYNC] [ObjectAdd] Object already exists.. ({})", add_object.path);
                let add_error_response = AddResponse {
                    object_id: object_id.to_vec(),
                    path: add_object.path,
                    success: false,
                    error: Some(AddError::AlreadySynced as i32)
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectAddResponse as u8], add_error_response)?;

                return Ok(())
            };
        }

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let object = Object {
            hostname: add_object.hostname,
            parent_tree: add_object.parent_tree,
            child_of_tree: add_object.child_of_tree,
            path: add_object.path.clone(),
            is_directory: add_object.is_directory,
            hash: add_object.hash,
            last_modified: timestamp,
            object_id: object_id.clone(),
            chunk_hashes: None,
            chunk_data: vec![],
        };

        println!("[*] [DSYNC] [ObjectAdd] Object: {:?}", object.path);

        let write_txn = database.begin_write()?;
        {
            let mut objects = write_txn.open_table(OBJECTS_TABLE)?;
            objects.insert(object_id.clone(), bitcode::encode(&object))?;
        }
        write_txn.commit()?;

        let add_response = AddResponse {
            object_id: object_id.to_vec(),
            path: add_object.path,
            success: true,
            error: None
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectAddResponse as u8], add_response)?;

        // Force a status update
        let status = Status {
            object_id: object_id.to_vec(),
            hash: None,
            modified_last: None
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectStatus as u8], status)?;

        Ok(())
    }
}