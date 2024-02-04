// Include the `protocol` module, which is generated from protocol.proto.
// It is important to maintain the same structure as in the proto.
mod protobuf {
    pub mod message {
        include!(concat!(env!("OUT_DIR"), "/protobuf.message.rs"));
    }
}

mod addr;
mod network;
mod pubsub;
mod router;
mod runner;
pub use protobuf::message::{protocol, Protocol};
pub use router::metric::Float;
pub use runner::{InputEvent, OutputEvent, P2pStreamRunner};
