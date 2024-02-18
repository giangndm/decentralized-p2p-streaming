use crate::{addr::NodeId, Float};

pub struct NetworkPkt {
    pub conn: Connection,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Connection(NodeId, u32);

impl Connection {
    pub fn from_parts(node: NodeId, session: u32) -> Self {
        Self(node, session)
    }

    pub fn remote(&self) -> NodeId {
        self.0
    }

    pub fn session(&self) -> u32 {
        self.1
    }
}

pub struct NetworkMsg<MSG> {
    pub conn: Connection,
    pub msg: MSG,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectionStats {
    pub rtt_ms: u32,
    pub lost_percent: Float<2>,
    pub jitter_ms: u32,
    pub bandwidth_kbps: u32,
}
