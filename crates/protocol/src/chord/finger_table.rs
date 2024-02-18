// Implement from https://pdos.csail.mit.edu/papers/ton:chord/paper-ton.pdf

use std::collections::{HashMap, VecDeque};

use crate::addr::NodeId;

const FINGER_TABLE_SIZE: usize = 32;
const REQUEST_TIMEOUT_MS: u64 = 10000;
const PREDECESSOR_WARMUP_MS: u64 = 10000;
const PREDECESSOR_TIMEOUT_MS: u64 = 10000;
type NodeAddr = String;

pub struct NodeIdRange(NodeId, NodeId);

impl NodeIdRange {
    pub fn new<A: Into<NodeId>>(start: A, end: A) -> Self {
        Self(start.into(), end.into())
    }
    pub fn contains(&self, key: NodeId, left: bool, right: bool) -> bool {
        if left && self.0 == key {
            return true;
        }
        if right && self.1 == key {
            return true;
        }

        if self.0 <= self.1 {
            if key < self.0 || key > self.1 {
                return false;
            }
            true
        } else {
            if key < self.1 || key > self.0 {
                return true;
            }
            false
        }
    }
}

#[derive(Debug)]
pub enum ChordEvent {
    PingPredecessor {
        remote: NodeId,
        ts: u64,
    },
    PongPredecessor {
        remote: NodeId,
        ts: u64,
    },
    Notify {
        remote: NodeId,
        info: NodeInfo,
    },
    FindSuccessor {
        req: u32,
        remote: NodeId,
        key: NodeId,
    },
    FindPredecessor {
        req: u32,
        remote: NodeId,
    },
    FoundSuccessor {
        req: u32,
        remote: NodeId,
        key: NodeId,
        info: Option<NodeInfo>,
    },
    FoundPredecessor {
        req: u32,
        remote: NodeId,
        info: Option<NodeInfo>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub address: NodeAddr,
    pub created_at: u64,
    pub last_pong: Option<u64>,
}

#[derive(Debug)]
struct RequestSlot {
    source: RequestSource,
    ts: u64,
}

#[derive(Debug)]
enum RequestSource {
    Remote { node: NodeId, key: NodeId, req: u32 },
    LocalSuccessor { slot: usize },
    LocalPredecessor,
}

#[derive(Debug)]
pub struct ChordFingerTable {
    req_seed: u32,
    node_info: NodeInfo,
    finger: [Option<NodeInfo>; FINGER_TABLE_SIZE],
    predecessor: Option<NodeInfo>,
    actions: VecDeque<ChordEvent>,
    req_queue: HashMap<u32, RequestSlot>,
    fix_finger_next: usize,
}

impl ChordFingerTable {
    pub fn new(node_info: NodeInfo) -> Self {
        let finger: [Option<NodeInfo>; FINGER_TABLE_SIZE] = Default::default();
        Self {
            req_seed: 0,
            node_info,
            finger,
            predecessor: None,
            actions: VecDeque::new(),
            req_queue: HashMap::new(),
            fix_finger_next: 0,
        }
    }

    pub fn join(&mut self, now_ms: u64, remote: NodeId) {
        self.predecessor = None;
        let req = self.next_req();
        self.actions.push_back(ChordEvent::FindSuccessor {
            req,
            remote,
            key: self.node_info.node_id,
        });
        self.req_queue.insert(
            req,
            RequestSlot {
                source: RequestSource::LocalSuccessor { slot: 0 },
                ts: now_ms,
            },
        );
    }

    pub fn on_event(&mut self, now_ms: u64, event: ChordEvent) {
        match event {
            ChordEvent::PingPredecessor { remote, ts } => {
                self.actions
                    .push_back(ChordEvent::PongPredecessor { remote, ts });
            }
            ChordEvent::PongPredecessor { remote, ts } => {
                if let Some(predecessor) = &mut self.predecessor {
                    if predecessor.node_id == remote {
                        predecessor.last_pong = Some(ts);
                    }
                }
            }
            ChordEvent::Notify { remote: _, info } => {
                if let Some(predecessor) = &self.predecessor {
                    if NodeIdRange(predecessor.node_id, self.node_info.node_id).contains(
                        info.node_id,
                        false,
                        false,
                    ) {
                        self.predecessor = Some(info);
                    }
                } else {
                    self.predecessor = Some(info);
                }
            }
            ChordEvent::FindSuccessor { req, remote, key } => {
                self.find_successor(
                    now_ms,
                    RequestSource::Remote {
                        node: remote,
                        key,
                        req,
                    },
                    key,
                );
            }
            ChordEvent::FindPredecessor { req, remote } => {
                self.actions.push_back(ChordEvent::FoundPredecessor {
                    req,
                    remote,
                    info: self.predecessor.clone(),
                });
            }
            ChordEvent::FoundSuccessor {
                req,
                remote: _,
                key,
                info,
            } => {
                //TODO validate remote node is correct or not?
                if let Some(slot) = self.req_queue.remove(&req) {
                    match slot.source {
                        RequestSource::Remote { node, key: _, req } => {
                            if node == self.node_info.node_id {
                                panic!("unexpected local request");
                            } else {
                                self.actions.push_back(ChordEvent::FoundSuccessor {
                                    req,
                                    remote: node,
                                    key,
                                    info,
                                });
                            }
                        }
                        RequestSource::LocalSuccessor { slot } => {
                            if let Some(info) = info {
                                self.finger[slot] = Some(info);
                            }
                        }
                        RequestSource::LocalPredecessor => {
                            panic!("wrong request source for FoundSuccessor event")
                        }
                    }
                }
            }
            ChordEvent::FoundPredecessor {
                req,
                remote: _,
                info,
            } => {
                //TODO validate remote node is correct or not?
                if let Some(slot) = self.req_queue.remove(&req) {
                    match slot.source {
                        RequestSource::Remote { .. } => {
                            panic!("wrong request source for FoundPredecessor event")
                        }
                        RequestSource::LocalSuccessor { .. } => {
                            panic!("wrong request source for FoundPredecessor event")
                        }
                        RequestSource::LocalPredecessor => {
                            if let Some(info) = info {
                                self.stablize_1(info);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn stablize(&mut self, now_ms: u64) {
        self.stablize_0(now_ms);
    }

    pub fn fix_fingers(&mut self, now_ms: u64) {
        self.fix_finger_next += 1;
        if self.fix_finger_next >= FINGER_TABLE_SIZE {
            self.fix_finger_next = 0;
        }
        self.find_successor(
            now_ms,
            RequestSource::LocalSuccessor {
                slot: self.fix_finger_next,
            },
            self.node_info
                .node_id
                .wrapping_add(2 << self.fix_finger_next as u32)
                .into(),
        );
    }

    pub fn check_predecessor(&mut self, now_ms: u64) {
        if let Some(predecessor) = &self.predecessor {
            if predecessor.created_at + PREDECESSOR_WARMUP_MS <= now_ms {
                if now_ms - predecessor.last_pong.unwrap_or(0) >= PREDECESSOR_TIMEOUT_MS {
                    self.predecessor = None;
                    return;
                }
            }
            self.actions.push_back(ChordEvent::PingPredecessor {
                remote: predecessor.node_id,
                ts: now_ms,
            });
        }
    }

    pub fn check_timeout_requests(&mut self, now_ms: u64) {
        let mut to_remove = vec![];
        for (req, slot) in &self.req_queue {
            if now_ms - slot.ts >= REQUEST_TIMEOUT_MS {
                to_remove.push(*req);
            }
        }
        for req in to_remove {
            if let Some(slot) = self.req_queue.remove(&req) {
                if let RequestSource::Remote { node, key, req } = slot.source {
                    self.actions.push_back(ChordEvent::FoundSuccessor {
                        req,
                        remote: node,
                        key,
                        info: None,
                    });
                }
            }
        }
    }

    pub fn pop_action(&mut self) -> Option<ChordEvent> {
        self.actions.pop_front()
    }

    /// successor is the first entry in the finger table or the node itself
    fn successor(&self) -> &NodeInfo {
        self.finger[0].as_ref().unwrap_or(&self.node_info)
    }

    fn find_successor(&mut self, now_ms: u64, source: RequestSource, key: NodeId) {
        let successor = self.successor();
        if NodeIdRange(self.node_info.node_id, successor.node_id).contains(key, false, true) {
            match source {
                RequestSource::Remote { node, key: _, req } => {
                    self.actions.push_back(ChordEvent::FoundSuccessor {
                        req,
                        remote: node,
                        key,
                        info: Some(successor.clone()),
                    });
                }
                RequestSource::LocalSuccessor { slot } => {
                    self.finger[slot] = Some(successor.clone());
                }
                RequestSource::LocalPredecessor => {
                    panic!("wrong request source for FoundSuccessor event")
                }
            }
        } else {
            let req = self.next_req();
            let next_node = self.closest_preceding_node(key).node_id;
            if next_node != self.node_info.node_id {
                self.actions.push_back(ChordEvent::FindSuccessor {
                    req,
                    remote: next_node,
                    key,
                });
                self.req_queue
                    .insert(req, RequestSlot { source, ts: now_ms });
            }
        }
    }

    fn closest_preceding_node(&self, key: NodeId) -> &NodeInfo {
        for i in (0..FINGER_TABLE_SIZE).rev() {
            if let Some(info) = &self.finger[i] {
                if NodeIdRange(self.node_info.node_id, key).contains(info.node_id, false, false) {
                    return info;
                }
            }
        }
        &self.node_info
    }

    fn next_req(&mut self) -> u32 {
        let req = self.req_seed;
        self.req_seed += 1;
        req
    }

    fn stablize_0(&mut self, now_ms: u64) {
        let req = self.next_req();
        let successor = self.successor();
        self.actions.push_back(ChordEvent::FindPredecessor {
            req,
            remote: successor.node_id,
        });
        self.req_queue.insert(
            req,
            RequestSlot {
                source: RequestSource::LocalPredecessor,
                ts: now_ms,
            },
        );
    }

    fn stablize_1(&mut self, predecessor: NodeInfo) {
        if NodeIdRange(self.node_info.node_id, self.successor().node_id).contains(
            predecessor.node_id,
            false,
            false,
        ) {
            self.finger[0] = Some(predecessor);
        }
        self.actions.push_back(ChordEvent::Notify {
            remote: self.successor().node_id,
            info: self.node_info.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::chord::finger_table::NodeIdRange;

    #[test]
    fn node_range() {
        assert_eq!(
            NodeIdRange::new(10, 20).contains(10.into(), true, true),
            true
        );
        assert_eq!(
            NodeIdRange::new(10, 20).contains(20.into(), true, true),
            true
        );
        assert_eq!(
            NodeIdRange::new(10, 20).contains(11.into(), true, true),
            true
        );
        assert_eq!(
            NodeIdRange::new(10, 20).contains(1.into(), true, true),
            false
        );

        assert_eq!(
            NodeIdRange::new(20, 10).contains(10.into(), true, true),
            true
        );
        assert_eq!(
            NodeIdRange::new(20, 10).contains(20.into(), true, true),
            true
        );
        assert_eq!(
            NodeIdRange::new(20, 10).contains(11.into(), true, true),
            false
        );
        assert_eq!(
            NodeIdRange::new(20, 10).contains(1.into(), true, true),
            true
        );
    }
}
