#!/usr/bin/env fish

# allow handling ICMP packets in user space
# Credit: <https://stackoverflow.com/questions/29496575/what-handles-ping-in-linux>

# ignore ICMP in kernel
sudo sysctl net.ipv4.icmp_echo_ignore_all=1
# allow using ICMP socket
sudo sysctl net.ipv4.ping_group_range='0 10000'
