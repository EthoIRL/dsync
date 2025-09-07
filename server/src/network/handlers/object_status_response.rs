use crate::config::Config;
use crate::network::handlers::object_sync::ObjectSync;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::comms::object::{StatusResponse, Sync};
use redb::{Database, ReadableDatabase};
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::handlers::object_add::Object;
use crate::network::packet;
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_TABLE;

pub struct ObjectStatusResponse;

impl GenericHandler for ObjectStatusResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>> {
        let status_response: StatusResponse = packet.decode()?;

        if status_response.object_id.len() < 4 || status_response.object_id.len() > 4 {
            return Err(format!("Invalid object_id length: ({})", status_response.object_id.len()).into())
        }

        let object_id: [u8; 4] = status_response.object_id[0..4].try_into()?;

        println!("Handling object status response");

        let state = ObjectState::try_from(status_response.state)?;

        match state {
            ObjectState::Fine => {
                return Ok(())
            },
            ObjectState::RemoteOutOfDate => {
                let read_txn = database.begin_read()?;
                let object_table = read_txn.open_table(OBJECTS_TABLE)?;

                match object_table.get(&object_id)? {
                    None => return Err("No object found?".into()),
                    Some(object) => {
                        let object: Object = bitcode::decode(&*object.value())?;

                        match object.chunk_hashes {
                            None => {
                                let sync_request = Sync {
                                    object_id: status_response.object_id.clone(),
                                };

                                packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;

                                Ok(())
                            },
                            Some(chunk_hashes) => {
                                todo!()
                            }
                        }
                    }
                }
            },
            ObjectState::LocalOutOfDate => {
                todo!()
            }
        }
    }
}