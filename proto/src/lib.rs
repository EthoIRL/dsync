pub use crate::packets::object_add::Add;
use num_enum::TryFromPrimitive;
use thiserror::Error;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod header;
mod packets;

#[derive(TryFromPrimitive)]
#[repr(u8)]
pub enum PacketId {
    ObjectAdd = 1,
}

pub enum Packet {
    ObjectAdd(Add),
}

pub trait PacketData: FromBytes + IntoBytes + KnownLayout + Immutable + 'static {
    const ID: PacketId;
    fn wrap(self) -> Packet;
}

#[derive(Error, Debug)]
pub enum PacketPaseError {
    #[error("Packet id does not exist ({0})")]
    InvalidPacketID(u8),
    #[error("Data size is smaller than expected ({0})")]
    InvalidPacketSize(u8),
    #[error("Failed to parse packet data into packet type")]
    FailedParse
}

impl Packet {
    pub fn parse(packet_id: u8, data: &[u8]) -> Result<Packet, PacketPaseError> {
        let id = PacketId::try_from(packet_id)
            .map_err(|_| PacketPaseError::InvalidPacketID(packet_id))?;

        match id {
            PacketId::ObjectAdd => parse_packet::<Add>(data),
        }
    }
}

fn parse_packet<T: PacketData>(data: &[u8]) -> Result<Packet, PacketPaseError> {
    if data.len() < size_of::<T>() {
        return Err(PacketPaseError::InvalidPacketSize(data.len() as u8));
    }

    let parsed_packet = T::read_from_bytes(data)
        .map_err(|_| PacketPaseError::FailedParse)?;

    Ok(parsed_packet.wrap())
}