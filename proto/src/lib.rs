#[macro_export]
macro_rules! impl_packet_data {
    ($struct_name:ident, $packet_id:ident) => {
        impl PacketData for $struct_name {
            const ID: u8 = PacketDiscriminants::$packet_id as u8;

            fn wrap(self) -> Packet {
                Packet::$packet_id(self)
            }

            fn unwrap(self) -> Self {
                self
            }

            fn id(&self) -> u8 {
                Self::ID
            }
        }
    };
}

pub use crate::packets::object_add::Add;
use aes::cipher::BlockEncrypt;
use aes::Aes256;
use cipher::block_padding::Pkcs7;
use std::net::TcpStream;
use strum_macros::EnumDiscriminants;
use strum_macros::FromRepr;
use thiserror::Error;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod header;
mod packets;

#[derive(Debug, EnumDiscriminants)]
#[repr(u8)]
#[strum_discriminants(derive(FromRepr))]
pub enum Packet {
    ObjectAdd(Add) = 1,
}

pub trait PacketData: FromBytes + IntoBytes + Immutable + KnownLayout + 'static {
    const ID: u8;
    fn wrap(self) -> Packet;
    fn unwrap(self) -> Self;
    fn id(&self) -> u8;
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
        let packet_discriminant = match PacketDiscriminants::from_repr(packet_id) {
            Some(discriminant) => discriminant,
            None => return Err(PacketPaseError::InvalidPacketID(packet_id))
        };

        match packet_discriminant {
            PacketDiscriminants::ObjectAdd => parse_packet::<Add>(data),
        }
    }

    pub fn send<T: PacketData>(stream: &mut TcpStream, packet: T, aes_cipher: &Option<Aes256>) -> Result<(), ()> {
        let internal_packet = packet.unwrap();
        let mut data = internal_packet
            .as_bytes()
            .to_vec();

        if let Some(cipher) = aes_cipher {
            data = cipher.encrypt_padded_vec::<Pkcs7>(&data);
        }

        header::send(stream, internal_packet.id(), data)
            .map_err(|_| ())
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