use crate::PacketDiscriminants;
use crate::{impl_packet_data, Packet, PacketData};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Add {
    pub data: u8,
    pub test: [u8; 48]
}

impl_packet_data!(Add, ObjectAdd);