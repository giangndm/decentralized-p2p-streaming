use std::collections::{HashMap, VecDeque};

use crate::network::Connection;

const SUB_TIMEOUT_MS: u64 = 5000;

struct RemoteSub {
    last_sub: u64,
}

pub enum OutputEvent {
    Sub,
    Data {
        data: Vec<u8>,
        remotes: Vec<Connection>,
        local: bool,
    },
    Unsub,
}

pub struct PubsubChannel {
    local_sub: bool,
    remote_subs: HashMap<Connection, RemoteSub>,
    outputs: VecDeque<OutputEvent>,
}

impl PubsubChannel {
    pub fn new() -> Self {
        Self {
            local_sub: false,
            remote_subs: HashMap::new(),
            outputs: VecDeque::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.local_sub || self.remote_subs.is_empty()
    }

    pub fn on_tick(&mut self, now_ms: u64) {
        //clear timeout remote subs
        let timeout = now_ms - SUB_TIMEOUT_MS;
        self.remote_subs.retain(|_, sub| sub.last_sub > timeout);

        if self.local_sub || !self.remote_subs.is_empty() {
            self.outputs.push_back(OutputEvent::Sub);
        }
    }

    pub fn relay_data(&mut self, data: Vec<u8>) {
        let remotes = self
            .remote_subs
            .iter()
            .map(|(conn, _)| *conn)
            .collect::<Vec<_>>();
        if !remotes.is_empty() || self.local_sub {
            self.outputs.push_back(OutputEvent::Data {
                data,
                remotes,
                local: self.local_sub,
            });
        }
    }

    pub fn on_local_sub(&mut self) {
        if !self.local_sub {
            self.local_sub = true;
            if self.remote_subs.is_empty() {
                self.outputs.push_back(OutputEvent::Sub);
            }
        }
    }

    pub fn on_local_unsub(&mut self) {
        if self.local_sub {
            self.local_sub = false;
            if self.remote_subs.is_empty() {
                self.outputs.push_back(OutputEvent::Unsub);
            }
        }
    }

    pub fn on_remote_sub(&mut self, now_ms: u64, from: Connection) {
        if let Some(remote) = self.remote_subs.get_mut(&from) {
            remote.last_sub = now_ms;
        } else {
            if !self.local_sub && self.remote_subs.is_empty() {
                self.outputs.push_back(OutputEvent::Sub);
            }
            self.remote_subs
                .insert(from, RemoteSub { last_sub: now_ms });
        }
    }

    pub fn on_remote_unsub(&mut self, _now_ms: u64, from: Connection) {
        if self.remote_subs.remove(&from).is_some()
            && self.remote_subs.is_empty()
            && !self.local_sub
        {
            self.outputs.push_back(OutputEvent::Unsub);
        }
    }

    pub fn pop_output(&mut self) -> Option<OutputEvent> {
        self.outputs.pop_front()
    }
}
