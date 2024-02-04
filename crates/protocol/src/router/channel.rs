use std::collections::HashMap;

use crate::{
    addr::{ChannelId, NodeId},
    network::Connection,
};

use super::path::ChannelPath;

pub struct ChannelRoute {
    channel_id: ChannelId,
    paths: HashMap<Connection, ChannelPath>,
}

impl ChannelRoute {
    pub fn new(channel_id: ChannelId) -> Self {
        Self {
            channel_id,
            paths: HashMap::new(),
        }
    }

    pub fn on_tick(&mut self, _now_ms: u64) {}

    pub fn on_sync(&mut self, _now_ms: u64, from: Connection, path: ChannelPath) {
        self.paths.insert(from, path);
    }

    pub fn on_disconnected(&mut self, conn: Connection) {
        self.paths.remove(&conn);
    }

    pub fn create_sync(&self, dest: NodeId) -> Option<ChannelPath> {
        //find the best path to the destination which hops not contains dest
        let mut best_path: Option<&ChannelPath> = None;
        for path in self.paths.values() {
            if !path.hops.contains(&dest) {
                if let Some(best) = best_path {
                    if path.metric.score() < best.metric.score() {
                        best_path = Some(path);
                    }
                } else {
                    best_path = Some(path);
                }
            }
        }
        best_path.map(|p| p.clone())
    }

    pub fn next_hop(&self) -> Option<Connection> {
        //find the best path to the destination which hops not contains dest
        let mut best: Option<(&Connection, &ChannelPath)> = None;
        for (conn, path) in &self.paths {
            if let Some((_, best_path)) = best {
                if path.metric.score() < best_path.metric.score() {
                    best = Some((conn, path));
                }
            } else {
                best = Some((conn, path));
            }
        }
        best.map(|p| *p.0)
    }
}
