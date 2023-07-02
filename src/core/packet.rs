use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PacketType {
    InPacket,
    OutPacket,
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Packet {
    Message {
        ty: PacketType,
        content: String,
    },
}
