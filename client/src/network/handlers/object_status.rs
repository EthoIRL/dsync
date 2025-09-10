use crate::config::Config;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::network::tools::{protofile, prototools};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::Sync;
use crate::proto::comms::object::{Status, StatusResponse};
use crate::proto::constant::PacketKind;
use redb::Database;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use xxhash_rust::xxh3::xxh3_64;

pub struct ObjectStatus;

impl GenericHandler for ObjectStatus {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let status: Status = packet.decode()?;

        let object_id = prototools::parse_object_id(&status.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        let object_state = match status.hash {
            None => {
                if !path.exists() {
                    ObjectState::Deleted
                } else {
                    ObjectState::RemoteOutOfDate
                }
            },
            Some(hash) => {
                if let Ok(current_object_hash) = protofile::hash_object(&path) {
                    if current_object_hash != hash {
                        let last_modified_timestamp = protofile::object_last_modified(&path)?;

                        match status.modified_last {
                            None => {
                                return Err("Hash exists on remote master but timestamp doesn't?".into())
                            }
                            Some(remote_timestamp) => {
                                if remote_timestamp >= last_modified_timestamp {
                                    ObjectState::LocalOutOfDate
                                } else {
                                    if !path.exists() {
                                        ObjectState::Deleted
                                    } else {
                                        ObjectState::RemoteOutOfDate
                                    }
                                }
                            }
                        }
                    } else {
                        ObjectState::Fine
                    }
                } else {
                    ObjectState::Fine
                }
            }
        };

        println!("[*] [DSYNC] [ObjectStatus] {:?} [State: {:#?}]", path, object_state);

        let response = StatusResponse {
            object_id: status.object_id.clone(),
            state: object_state as i32
        };

        packet::send_packet(stream, &mut [PacketKind::ObjectStatusResponse as u8], response)?;

        if object_state == ObjectState::LocalOutOfDate {
            let sync_request = Sync {
                object_id: status.object_id
            };

            packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
        }

        Ok(())
    }
}