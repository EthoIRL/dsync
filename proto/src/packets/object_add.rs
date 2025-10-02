use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};
use crate::{Packet, PacketData, PacketId};

#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Add {
    
}

impl PacketData for Add {
    const ID: PacketId = PacketId::ObjectAdd;
    fn wrap(self) -> Packet {
        Packet::ObjectAdd(self)
    }
}