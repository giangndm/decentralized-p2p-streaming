# atm0s decentraized p2p streaming

This is a streaming stack for building decentralized p2p streaming applications like video conference, live streaming, etc. The goal is to build the stack with minimum server usage and maximum p2p usage but still maintain the latency and quality of streaming. In theory, with p2p NATS traversal success rate is around 70%, it can reduce the cost of streaming service to zero if we have enough users.

## How it works

By using some ideas from original atm0s-sdn for create a decentralized p2p network with some protocol liked WebRTC, WebSocket.

The main idea is publisher best next hop is synced to all neighbours, and the each neighbours is continue to sync to their neighbours. So the subscriber can known the best next hop to connect to the publisher.

The details of protocol is in [rfc draft](./crates/protocol/docs/rfcs/2023-decentralized-p2p-streaming-01.md).

## Modules

- Native module: which is used in native application or running as relay server.
- Web module: which is used in web application.
- Protocol: define the protocol for p2p streaming.

## Status

This is in very early stage of development, so it's not ready for production. But you can try it out and give me some feedback.

## Check list

- [ ] Basic protocol, rfc draft (Working in progress)
- [ ] Native module with WebRTC
- [ ] Web module with WebRTC
- [ ] Audio streaming
- [ ] Video streaming
- [ ] High quality, Multi-layers Video streaming

## Roadmap

- [ ] Bootstrap: Implement first version of protocol with audio only (Opus)
- [ ] Optimize: Improving decentralized network topology and testing in real world
- [ ] Enhancing 1: Implement video streaming (AV1)
- [ ] Enhancing 2: Implement high quality, multi-layers video streaming (AV1)
