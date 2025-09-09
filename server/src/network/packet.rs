use std::io::{Error, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use prost::Message;
use redb::Database;
use crate::config::Config;

pub struct GenericPacket {
    pub id: u8,
    pub data: Vec<u8>
}

impl GenericPacket {
    pub fn decode<T: Message + Default>(&self) -> Result<T, Error> {
        T::decode(&*self.data).map_err(|err| Error::from(err))
    }
}

pub fn get_packet(stream: &mut TcpStream, packet_id: &mut [u8; 1], data_length_buffer: &mut [u8; 4]) -> Result<GenericPacket, Error> {
    stream.read_exact(packet_id)?;

    stream.read_exact(data_length_buffer)?;
    let data_length = u32::from_le_bytes(*data_length_buffer);

    let mut buffer = vec![0u8; data_length as usize];
    if data_length > 0 {
        stream.read_exact(&mut buffer)?;
    }

    Ok(GenericPacket {
        id: packet_id[0],
        data: buffer
    })
}

pub fn send_packet(stream: &mut TcpStream, packet_id: &mut [u8; 1], packet: impl Message) -> Result<(), Error> {
    let packet_buffer: Vec<u8> = packet.encode_to_vec();
    let packet_length = u32::to_le_bytes(packet_buffer.len() as u32);

    stream.write_all(packet_id)?;
    stream.write_all(&packet_length)?;
    
    if packet_buffer.len() > 0 {
        stream.write_all(&packet_buffer)?;
    }
    stream.flush()?;

    Ok(())
}

pub trait GenericHandler {
    fn handle(stream: &mut TcpStream, packet: GenericPacket, config: &Arc<Config>, database: &Arc<Database>) -> Result<(), Box<dyn std::error::Error>>;
}