# RFC for Decentralized Streaming Protocol

- Version: 0.1-draft
- Last update: 2024-01-01
- Author: giang.ndm@gmail.com (https://github.com/giangndm)

## 1. Introduction

### 1.1 Purpose

Today streaming application is used very widely, from video conference, live streaming, etc. But most of them are using centralized server to stream data to clients. This is not good for privacy and security and take a lot of cost for server. So we need a decentralized streaming protocol to solve this problem.

In this document, we will propose a solution for creating a decentralized streaming protocol which remain the latency and quality of streaming but reduce the cost of streaming service to zero if we have enough users.

### 1.2 Scope

This protocol is intended to be used for creating decentralized streaming applications like video conference, live streaming, etc. Which support audio and video streaming and also chat channel.

## 2. Terminology

In this document we will use some terms:

- **Publisher**: The node which is streaming data to other nodes.
- **Subscriber**: The node which is receiving data from publisher.
- **Relay**: The node which is used to relay data from publisher to subscriber.
- **Neighbour**: The node which is connected to other node.
- **Best next hop**: The best neighbour to connect to publisher.
- **Channel**: The channel which is used to stream data.
- **Stream**: Audio or video stream.
- **Packet**: Audio or video packet.

## 3. Protocol Overview

Asume that we already has a p2p network which connected all nodes together without any partion. We can use any of network topology like structed or unstructed, etc. Some network topology like Kademlia, Chord, etc. can be used to create a p2p network.

### 3.1 Connection cost function

The connection cost function is used to calculate the cost of connection between two nodes. The cost is used to find the best next hop to connect to publisher.

The cost function is calculated by:

```
cost = A * latency + B * current_usage + C * packet_loss + D * jitter + E
```

And above cost function need to same for all nodes in the network.

Periodically, each node will ping all neighbours to get the latency, packet loss, jitter, etc. Then the cost function will be calculated and saved to metrics table in each node:

| Neighbour | Cost |
|-----------|------|
| Node01    | 10   |
| Node02    | 15   |
| Node01    | 20   |

Note that we can have multiple connections between 2 nodes. The connection can be TCP, UDP, WebRTC, etc.

### 3.2 Router table

Router table is used to store the best next hop to connect to publisher. The router table is synced to all neighbours.

Each node will hold a router table like bellow:

| Channel   | Next hop | Cost |       Path       |
|-----------|----------|------|------------------|
| Channel01 | Node01   | 10   | [Node01, Node02] |
| Channel02 | Node02   | 15   | [Node02]         |
| Channel03 | Node03   | 20   | [Node03, Node02] |

### 3.3 Router sync

The router table is synced to all neigbour prediodically. The sync period default is 1 second.

The sync message is generate by bellow algorithm:

```
For each neighbour connection:
    Create new SYNC_MSG
    For each channel in router table:
        Select the best next hop for that channel which path not contain neighbour
        Append (channel, next_hop, connection, cost) to SYNC_MSG
    End for
    Send SYNC_MSG to neighbour
End for
```

### 3.4 Fast path prove

For proving, we will start with any incorrect state at beginning and prove that the state of network will become correct after some sync cycles.

Start with node which has publisher called root node, which will have only one rule: loop back with cost 0. The root node will send sync message to all neighbours. The neighbours will receive the sync message and update their router table. At that point, all direct neighbour will have correct router rule of direct connection to root node.

After that, each directed neighbour of the root node will continue sync router to all other its neighbours. The sync will be spread therefore all nodes will be synced after some sync cycles.

And with above sync message build logic, it also avoid the loop in the network.

### 3.5 Stream data flow

By use above router table, from subscribe it will send SUB request to it self. Each time a node receive SUB command, it will check if already has RELAY for that channel and simple add sender as relay destinations, if don't have RELAY for that channel it simple create RELAY and send SUB to next hop.

By that way, we can build a optimal tree of RELAY for each channel.

For avoiding waste bandwidth, each RELAY will periodically send SUB to next hop if it still has subscriber. If it don't have any subscriber, it will send UNSUB to next hop and remove itself. RELAY also remove destination if it don't receive SUB from destination after timeout.

## 4. Protocol Details

### 4.1 Protocol Messages

For adapt with different programing language, we will use protobuf to define protocol messages.

SYNC_MSG:
```
```

SUB:
```
```

UNSUB:
```
```

DATA:
```
```

### 4.2 Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| SYNC_INTERVAL | Sync route interval |    1s     |
| SUB_INTERVAL  | Re-sub interval  |    1s     |
| SUB_TIMEOUT  | Subscribe timeout  |    5s     |

## 5. Performance Considerations

Discuss any potential performance issues and how they can be mitigated.

## 6. Security Considerations

Discuss any potential security issues and how they can be mitigated.

## 7. References

List any references or resources used in the creation of this document.

## 8. Acknowledgements

We thank the following people and organization for their contributions and support to this project:

OWS Vietnam,
Bluesea Network,
8xFF Foundation,
