use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;
use redb::{Database, TableDefinition};
use crate::config::Config;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::AddResponse;

pub const OBJECTS_LOCAL_TABLE: TableDefinition<[u8; 4], String> = TableDefinition::new("object_paths");

pub struct ObjectAddResponse;

impl GenericHandler for ObjectAddResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn Error>> {
        let add_response: AddResponse = packet.decode()?;

        println!("Test {:#?}", add_response);

        if !add_response.success {
            return match add_response.error {
                None => Err("AddResponse not returning a correct error message, when erroring.".into()),
                Some(err) => Err(format!("Error occurred during add response: {:#?}", AddError::try_from(err).unwrap()).into())
            }
        }

        if add_response.object_id.len() > 4 {
            return Err(format!("Object ID is larger than expected? ({})", add_response.object_id.len()).into());
        }

        todo!()
    }
}
