use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::RemoveResponse;
use redb::Database;
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use crate::network::tools::prototools;
use crate::tables::OBJECTS_LOCAL_TABLE;

pub struct ObjectRemoveResponse;

impl GenericHandler for ObjectRemoveResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, _: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let remove_response: RemoveResponse = packet.decode()?;

        let object_id = prototools::parse_object_id(&remove_response.object_id)?;
        let path = prototools::get_object_path(&object_id, &database)?;

        if !remove_response.success {
            println!("[*] [DSYNC] Failed to remove object [{}]->({})", prototools::object_id_hex(&object_id), path.display());
            return Ok(())
        }

        let write_txn = database.begin_write()?;
        {
            let mut object_table = write_txn.open_table(OBJECTS_LOCAL_TABLE)?;
            object_table.remove(&object_id)?;
        }
        write_txn.commit()?;

        println!("[*] [DSYNC] Successfully removed object [{}]->({})", prototools::object_id_hex(&object_id), path.display());

        Ok(())
    }
}