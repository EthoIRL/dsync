use crate::config::Config;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::{Status, StatusResponse, Sync};
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use xxhash_rust::xxh3::xxh3_64;

pub struct ObjectStatus;

impl GenericHandler for ObjectStatus {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let status: Status = packet.decode()?;

        if status.object_id.len() < 4 || status.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", status.object_id.len()).into())
        }

        let object_id: [u8; 4] = status.object_id[0..4].try_into()?;

        let read_txn = database.begin_read()?;
        let object_table = read_txn.open_table(OBJECTS_TABLE)?;

        match object_table.get(&object_id)? {
            None => return Err("No object found?".into()),
            Some(object) => {
                let object: Object = bitcode::decode(&*object.value())?;

                if object.is_directory {
                    todo!()
                }

                let object_hash = xxh3_64(&object.chunk_data);

                if object.hash != object_hash {
                    todo!("Internal hash miss-match, server miss recalculation after updating data somewhere! This is bad!")
                }

                let object_state = match status.hash {
                    // TODO: Rename ObjectState LocalOutOfDate to ClientOutOfDate, and RemoteOutOfDate to MasterOutOfDate. Very loose naming scheme atm
                    None => ObjectState::LocalOutOfDate,
                    Some(hash) => {
                        if hash == object_hash {
                            ObjectState::Fine
                        } else {
                            match status.modified_last {
                                None => {
                                    return Err("Hash exists on client but timestamp doesn't?".into())
                                }
                                Some(remote_timestamp) => {
                                    if remote_timestamp >= object.last_modified {
                                        ObjectState::LocalOutOfDate
                                    } else {
                                        ObjectState::RemoteOutOfDate
                                    }
                                }
                            }
                        }
                    }
                };

                println!("STATUS | PATH: {:#?}, STATE {:?}", object.path, object_state);

                let response = StatusResponse {
                    object_id: status.object_id.clone(),
                    state: object_state as i32
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectStatusResponse as u8], response)?;

                if object_state == ObjectState::RemoteOutOfDate {
                    let sync_request = Sync {
                        object_id: status.object_id
                    };

                    packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
                }
            }
        }


        Ok(())
    }
}