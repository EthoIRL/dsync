use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::{Sync, StatusResponse};
use redb::Database;
use std::error::Error;
use std::fs;
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::packet;
use crate::network::tools::prototools;
use crate::proto::comms::object::status_response::ObjectState;
use crate::proto::constant::PacketKind;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectStatusResponse;

impl GenericHandler for ObjectStatusResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let status_response: StatusResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&status_response.object_id)?;

        let state = ObjectState::try_from(status_response.state)?;

        println!("[*] [DSYNC] [StatusResponse] [{}] (State: {:#?})", prototools::object_id_hex(&object_id), state);

        match state {
            ObjectState::LocalOutOfDate => {
                let sync_request = Sync {
                    object_id: status_response.object_id.clone(),
                };

                packet::send_packet(stream, &mut [PacketKind::ObjectSync as u8], sync_request)?;
            },
            ObjectState::Deleted => {
                println!("[*] [DSYNC] Object deleted from the master [{}]", prototools::object_id_hex(&object_id));

                let path = prototools::get_object_path(&object_id, &database)?;

                if path.exists() {
                    fs::remove_dir_all(path)?;
                }

                let write_txn = database.begin_write()?;
                let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
                object_table.remove(&object_id)?;
            }
            _ => ()
        }

        Ok(())
    }
}