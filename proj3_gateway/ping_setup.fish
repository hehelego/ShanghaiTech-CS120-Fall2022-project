#!/usr/bin/env fish

# allow handling ICMP packets in user space
# Credit: <https://stackoverflow.com/questions/29496575/what-handles-ping-in-linux>

# reset iptables
sudo iptables -P INPUT ACCEPT
sudo iptables -P FORWARD ACCEPT
sudo iptables -P OUTPUT ACCEPT
sudo iptables -t nat -F
sudo iptables -t mangle -F
sudo iptables -F
sudo iptables -X

# ignore ICMP in kernel
sudo sysctl net.ipv4.icmp_echo_ignore_all=1

# allow using ICMP socket
sudo sysctl net.ipv4.ping_group_range='0 10000'
