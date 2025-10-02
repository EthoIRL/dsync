use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub struct GenericPacket {
    pub id: u8,
    pub data: Vec<u8>
}

impl GenericPacket {
    pub fn decode<T: FromBytes + Immutable + KnownLayout>(&self) -> Result<&T, Error> {
        match T::ref_from_bytes(&self.data) {
            Ok(value) => Ok(value),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Failed to deserialize network packet"))
        }
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

pub fn send_packet(stream: &mut TcpStream, packet_id: &mut [u8; 1], packet: impl IntoBytes + Immutable + KnownLayout) -> Result<(), Error> {
    let packet_buffer: &[u8] = packet.as_bytes();
    let packet_length = u32::to_le_bytes(packet_buffer.len() as u32);

    stream.write_all(packet_id)?;
    stream.write_all(&packet_length)?;

    if packet_buffer.len() > 0 {
        stream.write_all(&packet_buffer)?;
    }

    stream.flush()?;

    Ok(())
}