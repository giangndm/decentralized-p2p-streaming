use std::collections::{HashMap, VecDeque};

use crate::{
    addr::{ChannelId, NodeId},
    network::{Connection, ConnectionStats, NetworkMsg},
    protocol::network_message::MessageType,
    pubsub::{self, Pubsub},
    router::{self, NextHop, Router},
};

pub enum InputEvent {
    Recv(NetworkMsg<MessageType>),
    Stats(NetworkMsg<ConnectionStats>),
}

pub enum OutputEvent {
    Send(NetworkMsg<MessageType>),
    OnChannelData(ChannelId, Vec<u8>),
}

pub struct P2pStreamRunner {
    router: Router,
    pubsub: Pubsub,
    remote_channels: HashMap<ChannelId, Connection>,
    outputs: VecDeque<OutputEvent>,
}

impl P2pStreamRunner {
    pub fn new(node: NodeId) -> Self {
        Self {
            router: Router::new(node),
            pubsub: Pubsub::new(),
            remote_channels: HashMap::new(),
            outputs: VecDeque::new(),
        }
    }

    pub fn on_tick(&mut self, now_ms: u64) {
        self.router.on_tick(now_ms);
        self.pubsub.on_tick(now_ms);

        self.pop_router_outputs();
        self.pop_pubsub_outputs();
    }

    pub fn on_msg(&mut self, now_ms: u64, event: InputEvent) {
        match event {
            InputEvent::Stats(msg) => {
                self.router
                    .on_event(now_ms, router::InputEvent::ConnectionStats(msg));
            }
            InputEvent::Recv(NetworkMsg { conn, msg }) => match msg {
                MessageType::RouterSync(sync) => {
                    self.router.on_event(
                        now_ms,
                        router::InputEvent::Recv(NetworkMsg { conn, msg: sync }),
                    );
                }
                MessageType::ChannelSub(sub) => {
                    self.pubsub.on_event(
                        now_ms,
                        pubsub::InputEvent::RecvSub(NetworkMsg { conn, msg: sub }),
                    );
                    self.pop_pubsub_outputs();
                }
                MessageType::ChannelUnsub(unsub) => {
                    self.pubsub.on_event(
                        now_ms,
                        pubsub::InputEvent::RecvUnsub(NetworkMsg { conn, msg: unsub }),
                    );
                    self.pop_pubsub_outputs();
                }
                MessageType::ChannelData(data) => {
                    self.pubsub.on_event(
                        now_ms,
                        pubsub::InputEvent::RecvData(NetworkMsg { conn, msg: data }),
                    );
                    self.pop_pubsub_outputs();
                }
            },
        }
    }

    fn pop_router_outputs(&mut self) {
        let sync_msgs = self.router.create_sync();
        for sync in sync_msgs {
            self.outputs.push_back(OutputEvent::Send(NetworkMsg {
                conn: sync.conn,
                msg: MessageType::RouterSync(sync.msg),
            }));
        }
    }

    fn pop_pubsub_outputs(&mut self) {
        while let Some(event) = self.pubsub.pop_output() {
            match event {
                pubsub::OutputEvent::SendSub(sub) => {
                    let channel_id = sub.channel.into();
                    let conn = if let Some(conn) = self.remote_channels.get(&channel_id) {
                        *conn
                    } else {
                        if let Some(NextHop::Remote(conn)) = self.router.next_hop_for(channel_id) {
                            conn
                        } else {
                            continue;
                        }
                    };
                    self.outputs.push_back(OutputEvent::Send(NetworkMsg {
                        conn,
                        msg: MessageType::ChannelSub(sub),
                    }));
                }
                pubsub::OutputEvent::SendUnsub(unsub) => {
                    let channel_id = unsub.channel.into();
                    if let Some(conn) = self.remote_channels.remove(&channel_id) {
                        self.outputs.push_back(OutputEvent::Send(NetworkMsg {
                            conn,
                            msg: MessageType::ChannelUnsub(unsub),
                        }));
                    }
                }
                pubsub::OutputEvent::SendData(NetworkMsg { conn, msg }) => {
                    self.outputs.push_back(OutputEvent::Send(NetworkMsg {
                        conn,
                        msg: MessageType::ChannelData(msg),
                    }));
                }
                pubsub::OutputEvent::OnChannelData(channel_id, data) => self
                    .outputs
                    .push_back(OutputEvent::OnChannelData(channel_id, data)),
            }
        }
    }
}
