#!/usr/bin/env fish

sudo iptables -t filter -I OUTPUT -p tcp --tcp-flags RST RST -j DROP
