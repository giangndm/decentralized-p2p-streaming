use crate::{network::ConnectionStats, protocol::RouterRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Float<const ACC: u8> {
    value: u32,
}

impl<const ACC: u8> From<f32> for Float<ACC> {
    fn from(value: f32) -> Self {
        Self {
            value: (value * 10.0_f32.powi(ACC as i32)) as u32,
        }
    }
}

impl<const ACC: u8> Into<f32> for Float<ACC> {
    fn into(self) -> f32 {
        self.value as f32 / 10.0_f32.powi(ACC as i32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metric {
    pub rtt: u32,
    pub loss: Float<2>,
    pub jitter: u32,
    pub bandwidth: u32,
}

impl std::ops::Add<Metric> for Metric {
    type Output = Metric;

    fn add(self, other: Metric) -> Metric {
        Metric {
            rtt: self.rtt + other.rtt,
            loss: loss_plus(self.loss.into(), other.loss.into()).into(),
            jitter: self.jitter + other.jitter,
            bandwidth: self.bandwidth.min(other.bandwidth),
        }
    }
}

impl From<RouterRow> for Metric {
    fn from(value: RouterRow) -> Self {
        Self {
            rtt: value.rtt,
            loss: value.loss.into(),
            jitter: value.jitter,
            bandwidth: value.bandwidth,
        }
    }
}

impl Metric {
    pub fn score(&self) -> u32 {
        self.rtt
    }

    pub fn add_local(&self, stats: &ConnectionStats) -> Metric {
        let add = Metric {
            rtt: stats.rtt_ms as u32,
            loss: stats.lost_percent,
            jitter: stats.jitter_ms as u32,
            bandwidth: stats.bandwidth_kbps,
        };
        *self + add
    }
}

fn loss_plus(l1: f32, l2: f32) -> f32 {
    (1.0 - (1.0 - l1 / 100.0) * (1.0 - l2 / 100.0)) * 100.0
}
