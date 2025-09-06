use std::error::Error;
use std::net::TcpStream;
use redb::TableDefinition;
use crate::network::packet::{GenericHandler, GenericPacket};
use crate::proto::comms::object::add_response::AddError;
use crate::proto::comms::object::AddResponse;


pub struct ObjectAddResponse;

impl GenericHandler for ObjectAddResponse {
    fn handle(stream: &mut TcpStream, packet: GenericPacket) -> Result<(), Box<dyn Error>> {
        let add_response: AddResponse = packet.decode()?;

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
