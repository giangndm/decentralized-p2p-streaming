use crate::{
    addr::{ChannelId, NodeId},
    protocol::RouterRow,
};

use super::metric::Metric;

#[derive(Debug, Clone)]
pub struct ChannelPath {
    pub last_sync: u64,
    pub metric: Metric,
    pub hops: Vec<NodeId>,
}

impl ChannelPath {
    pub fn to_row(&self, channel: ChannelId) -> RouterRow {
        RouterRow {
            channel: *channel,
            rtt: self.metric.rtt,
            loss: self.metric.loss.into(),
            jitter: self.metric.jitter,
            bandwidth: 0,
            hops: self
                .hops
                .clone()
                .into_iter()
                .map(|n| *n)
                .collect::<Vec<u32>>(),
        }
    }

    pub fn from_row(now_ms: u64, value: RouterRow) -> Self {
        Self {
            last_sync: now_ms,
            hops: value.hops.iter().map(|n| (*n).into()).collect(),
            metric: value.clone().into(),
        }
    }
}
