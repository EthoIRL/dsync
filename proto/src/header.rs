use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;
use zerocopy::{FromBytes, Immutable, KnownLayout};

pub struct HeaderPacket {
    pub id: u8,
    pub data: Vec<u8>
}

impl HeaderPacket {
    pub fn decode<T: FromBytes + Immutable + KnownLayout>(&self) -> Result<&T, Error> {
        match T::ref_from_bytes(&self.data) {
            Ok(value) => Ok(value),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Failed to deserialize network packet"))
        }
    }
}

pub fn get(stream: &mut TcpStream, packet_id: &mut [u8; 1], data_length_buffer: &mut [u8; 4]) -> Result<HeaderPacket, Error> {
    stream.read_exact(packet_id)?;

    stream.read_exact(data_length_buffer)?;
    let data_length = u32::from_le_bytes(*data_length_buffer);

    let mut buffer = vec![0u8; data_length as usize];
    if data_length > 0 {
        stream.read_exact(&mut buffer)?;
    }

    Ok(HeaderPacket {
        id: packet_id[0],
        data: buffer
    })
}

pub fn send(stream: &mut TcpStream, packet_id: u8, packet_data: Vec<u8>) -> Result<(), Error> {
    let packet_length = u32::to_le_bytes(packet_data.len() as u32);

    stream.write_all(&[packet_id])?;
    stream.write_all(&packet_length)?;

    if packet_data.len() > 0 {
        stream.write_all(&packet_data)?;
    }

    stream.flush()?;

    Ok(())
}