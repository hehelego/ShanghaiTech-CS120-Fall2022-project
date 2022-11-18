# note on project 3

## design

### overview

#### layering model of internal node

- Application: Athernet applications
- Transport: Athernet ICMP/UDP/TCP socket
- Network: Athernet network layer process
- Data Link: Athernet MAC+PHY

#### layering model of gateway node

- Application: Internet applications
- Transport: TCP/IP + UDP/IP + ICMP/IP socket
- Netework: OS IP module, Athernet network layer process
- Data Link: Ethernet/WiFi, Athernet MAC+PHY

#### AIP network packet

A packet on Athernet network layer contains

- Source IP address
- Destination IP address
- Payload packet protocol identifier
- Payload packet, one of:
  - An ICMP packet: `IcmpPacket`
  - A UDP packet: `UdpPacket`
  - A TCP packet: `TcpPacket`

### internal node

#### Athernet socket object

An Athernet socket object contains:

- An unix domain socket for communication with Athernet network provider.
- Socket type identifier: `TCP, UDP, ICMP`
- Bind address: `SocketAddrV4`
- Received packets queue.
- Other states (e.g. TCP connection states, ICMP sequence number)

The socket objects do not directly uses Athernet MAC/PHY for I/O.
Instead, it use an unix domain socket to communicate with the Athernet IP network provider.

Behavior:

- On construction (with `type`, `port`):
  Create an unix domain socket and send `BindSocket` to AIP provider,
  wait for correct response to finish construction.
- On dropping:
  Send `UnbindSocket` packet via unix domain socket.
  Delete the unix domain socket file.
- Repeatedly receive packets from the unix domain socket and push them into the received packets queue.
- On send, send `SendPacket` to AIP provider to schedule packet sending.
- On recv, pop a packet from the received packets queue.
- Other state transitions

#### network layer

The network layer is able to:

- fragment the network packet into MAC packets
- deliver fragments of a packet to another node via MAC:
  in order, without interleaving with fragments of other network packets.
- reassembling the received fragments to a network packet.

#### service provider

On every Athernet node.
We use an UNIX domain socket server to provide network layer service:

Protocol incoming packets:

- `BindSocket(type, port)`: `type` can be `ICMP, UDP, TCP` and `port` is of type `u16`
- `UnbindSocket`
- `SendPacket(Packet)`

Protocol outgoing packets:

- `PacketReceived(Packet)`

Behavior:

- Listen on a specific on `/tmp/athernet_ip.sock`.
- Maintain a port-process mapping:
  - Map `tcp_socks: port <-> unix_domain_socket` for all TCP sockets.
  - Map `udp_socks: port <-> unix_domain_socket` for all UDP sockets.
  - List `icmp_socks: [unix_domain_socket]` for all ICMP sockets.
- On receiving `BindSocket(type, port)` from `sock_addr`:  
  - If `type` is `TCP` or `UDP`,  
    try to add the mapping `port -> sock_addr` in `bind_socks`,  
    send back `[0; 1]` on success or `[0xff; 1]` on error.  
  - If `type` is `ICMP`,  
    add `sock_addr` into `icmp_socks`.
- On receiving `UnbindSocket()` from `sock_addr`:  
  - Find the mapping pair `(port, sock_addr)` and remove it for TCP/UDP socket.
  - Remove `sock_addr` from `icmp_socks` list.
- On receiving `SendPacket(Packet)` from `sock_addr`,  
  1. Lookup `sock_addr` in the mapping to determine the socket type denoted by `sock_type`.
  2. Check if the type of `Packet` matches with `sock_type`.
  3. Schedule to send `Packet` via Athernet MAC.
- On receiving `Packet` from Athernet MAC,
  - For `ICMP` packet, send the packet to all `icmp_socks`
  - For `TCP` and `UDP` packet,
    1. extract the destination port number
    2. lookup `tcp_socks` or `udp_socks`
    3. send the packet to the corresponding unix domain socket

### gateway node

## references

- [an email regarding the usage of ICMP sockets on LWN.net](https://lwn.net/Articles/422330/)
- [What handles ping in linux? - Stack Overflow](https://stackoverflow.com/questions/29496575/what-handles-ping-in-linux)
- [Unprivileged ICMP sockets on Linux](https://sturmflut.github.io/linux/ubuntu/2015/01/17/unprivileged-icmp-sockets-on-linux/)
