use crate::{
    addr::{ChannelId, NodeId},
    network::{Connection, ConnectionStats, NetworkMsg},
    protocol::{self, RouterRow, RouterSync},
};
use std::collections::HashMap;

use self::{channel::ChannelRoute, path::ChannelPath};

mod channel;
pub mod metric;
mod path;

pub enum InputEvent {
    Recv(NetworkMsg<protocol::RouterSync>),
    ConnectionDisconnected(Connection),
    ConnectionStats(NetworkMsg<ConnectionStats>),
}

pub enum NextHop {
    Local,
    Remote(Connection),
}

pub struct Router {
    node: NodeId,
    conns: HashMap<Connection, ConnectionStats>,
    remote_channels: HashMap<ChannelId, ChannelRoute>,
    local_channels: HashMap<ChannelId, ()>,
}

impl Router {
    pub fn new(node: NodeId) -> Self {
        Self {
            node,
            conns: HashMap::new(),
            remote_channels: HashMap::new(),
            local_channels: HashMap::new(),
        }
    }

    pub fn node(&self) -> NodeId {
        self.node
    }

    pub fn add_channel(&mut self, channel: ChannelId) {
        self.local_channels.insert(channel, ());
    }

    pub fn remove_channel(&mut self, channel: ChannelId) {
        self.local_channels.remove(&channel);
    }

    pub fn on_tick(&mut self, now_ms: u64) {
        for channel in self.remote_channels.values_mut() {
            channel.on_tick(now_ms);
        }
    }

    pub fn on_event(&mut self, now_ms: u64, event: InputEvent) {
        match event {
            InputEvent::Recv(msg) => {
                let NetworkMsg { conn, msg } = msg;
                for row in msg.rows {
                    if let Some(channel) = self.remote_channels.get_mut(&(row.channel.into())) {
                        if let Some(stats) = self.conns.get(&conn) {
                            let mut path = ChannelPath::from_row(now_ms, row);
                            path.metric = path.metric.add_local(stats);
                            path.hops.push(conn.remote());
                            channel.on_sync(now_ms, conn, path);
                        }
                    } else {
                        log::warn!("Unknown channel {}", row.channel);
                    }
                }
            }
            InputEvent::ConnectionDisconnected(conn) => {
                self.conns.remove(&conn);
                for channel in self.remote_channels.values_mut() {
                    channel.on_disconnected(conn);
                }
            }
            InputEvent::ConnectionStats(stats) => {
                let NetworkMsg { conn, msg } = stats;
                self.conns.insert(conn, msg);
            }
        }
    }

    pub fn next_hop_for(&self, channel: ChannelId) -> Option<NextHop> {
        if self.local_channels.contains_key(&channel) {
            Some(NextHop::Local)
        } else {
            self.remote_channels
                .get(&channel)
                .and_then(|c| c.next_hop().map(NextHop::Remote))
        }
    }

    /// Create sync messages for all channels
    /// Each sync message contains the best path for the channel without relaying over destination node
    /// If local has channel, it will be included in the sync message, if not it will check remote channels
    pub fn create_sync(&self) -> Vec<NetworkMsg<RouterSync>> {
        let mut outputs = vec![];
        for conn in self.conns.keys() {
            let mut rows = vec![];
            for (id, _) in self.local_channels.iter() {
                rows.push(RouterRow {
                    channel: **id,
                    bandwidth: 10_000_000, //10Gbps
                    rtt: 0,
                    loss: 0.0,
                    jitter: 0,
                    hops: vec![],
                });
            }

            for (id, channel) in self.remote_channels.iter() {
                if self.local_channels.contains_key(id) {
                    continue;
                }
                if let Some(row) = channel.create_sync(conn.remote()) {
                    rows.push(row.to_row(*id));
                }
            }
            if !rows.is_empty() {
                outputs.push(NetworkMsg {
                    conn: conn.clone(),
                    msg: protocol::RouterSync { rows },
                });
            }
        }
        outputs
    }
}
