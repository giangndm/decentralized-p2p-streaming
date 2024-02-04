use std::collections::{HashMap, VecDeque};

use crate::{
    addr::ChannelId,
    network::NetworkMsg,
    protocol::{ChannelData, ChannelSub, ChannelUnsub},
};

use self::channel::PubsubChannel;

mod channel;

pub enum InputEvent {
    RecvSub(NetworkMsg<ChannelSub>),
    RecvData(NetworkMsg<ChannelData>),
    RecvUnsub(NetworkMsg<ChannelUnsub>),
}

pub enum OutputEvent {
    SendSub(ChannelSub),
    SendData(NetworkMsg<ChannelData>),
    SendUnsub(ChannelUnsub),
    OnChannelData(ChannelId, Vec<u8>),
}

pub struct Pubsub {
    outputs: VecDeque<OutputEvent>,
    channels: HashMap<ChannelId, PubsubChannel>,
}

impl Pubsub {
    pub fn new() -> Self {
        Self {
            outputs: VecDeque::new(),
            channels: HashMap::new(),
        }
    }

    pub fn sub_channel(&mut self, channel_id: ChannelId) {
        let channel = self
            .channels
            .entry(channel_id)
            .or_insert_with(PubsubChannel::new);
        channel.on_local_sub();
        Self::pop_channel_output(channel_id, channel, &mut self.outputs);
    }

    pub fn unsub_channel(&mut self, channel_id: ChannelId) {
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.on_local_unsub();
            Self::pop_channel_output(channel_id, channel, &mut self.outputs);
            if channel.is_empty() {
                self.channels.remove(&channel_id);
            }
        }
    }

    pub fn pub_channel(&mut self, channel_id: ChannelId, data: Vec<u8>) {
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.relay_data(data);
            Self::pop_channel_output(channel_id, channel, &mut self.outputs);
        }
    }

    pub fn on_tick(&mut self, now_ms: u64) {
        for (channel_id, channel) in &mut self.channels {
            channel.on_tick(now_ms);
            Self::pop_channel_output(*channel_id, channel, &mut self.outputs);
        }
    }

    pub fn on_event(&mut self, now_ms: u64, event: InputEvent) {
        match event {
            InputEvent::RecvSub(msg) => {
                let channel_id = msg.msg.channel;
                let channel = self
                    .channels
                    .entry(channel_id.into())
                    .or_insert_with(PubsubChannel::new);
                channel.on_remote_sub(now_ms, msg.conn);
                Self::pop_channel_output(channel_id.into(), channel, &mut self.outputs);
            }
            InputEvent::RecvData(msg) => {
                let channel_id = msg.msg.channel.into();
                if let Some(channel) = self.channels.get_mut(&channel_id) {
                    channel.relay_data(msg.msg.data);
                    Self::pop_channel_output(channel_id, channel, &mut self.outputs);
                }
            }
            InputEvent::RecvUnsub(msg) => {
                let channel_id = msg.msg.channel.into();
                if let Some(channel) = self.channels.get_mut(&channel_id) {
                    channel.on_remote_unsub(now_ms, msg.conn);
                    Self::pop_channel_output(channel_id, channel, &mut self.outputs);
                    if channel.is_empty() {
                        self.channels.remove(&channel_id);
                    }
                }
            }
        }
    }

    pub fn pop_output(&mut self) -> Option<OutputEvent> {
        self.outputs.pop_front()
    }

    fn pop_channel_output(
        channel_id: ChannelId,
        channel: &mut PubsubChannel,
        outputs: &mut VecDeque<OutputEvent>,
    ) {
        if let Some(output) = channel.pop_output() {
            match output {
                channel::OutputEvent::Sub => {
                    outputs.push_back(OutputEvent::SendSub(ChannelSub {
                        channel: *channel_id,
                    }));
                }
                channel::OutputEvent::Data {
                    data,
                    remotes,
                    local,
                } => {
                    for conn in remotes {
                        outputs.push_back(OutputEvent::SendData(NetworkMsg {
                            conn,
                            msg: ChannelData {
                                channel: *channel_id,
                                data: data.clone(),
                            },
                        }));
                    }
                    if local {
                        outputs.push_back(OutputEvent::OnChannelData(channel_id, data));
                    }
                }
                channel::OutputEvent::Unsub => {
                    outputs.push_back(OutputEvent::SendUnsub(ChannelUnsub {
                        channel: *channel_id,
                    }));
                }
            }
        }
    }
}
