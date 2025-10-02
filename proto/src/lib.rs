use crate::packets::object_add::Add;

pub mod packet;
mod packets;

#[repr(u8)]
pub enum PacketType {
    ObjectAdd(Add) = 1,
}