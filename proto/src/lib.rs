#[macro_export]
macro_rules! impl_packet_data {
    ($struct_name:ident, $packet_id:ident) => {
        impl PacketData for $struct_name {
            const ID: PacketId = PacketId::$packet_id;

            fn wrap(self) -> Packet {
                Packet::$packet_id(self)
            }

            fn unwrap(self) -> Self {
                self
            }

            fn id(&self) -> PacketId {
                Self::ID
            }
        }
    };
}

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

pub trait PacketData: FromBytes + IntoBytes + Immutable + KnownLayout + 'static {
    const ID: PacketId;
    fn wrap(self) -> Packet;
    fn unwrap(self) -> Self;
    fn id(&self) -> PacketId;
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