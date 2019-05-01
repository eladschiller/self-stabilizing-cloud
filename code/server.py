#!/bin/python3.6
# -*- coding: utf-8 -*-
#
# MIT License
#
# Copyright (c) 2018 Robert Gustafsson
# Copyright (c) 2018 Andreas Lindhé
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

import asyncio
import struct
import sys
import configparser
import math
import random as r
from channel.ppProtocol import PingPongMessage
from channel.GossipProtocol import GossipMessage
from channel.serverRecvChannel import ServerRecvChannel
from channel.SenderChannel import SenderChannel
from gossip.Gossip import Gossip
from quorum.QuorumRecv import QuorumRecv
from cas.SelfStabilizing import Server

def get_uid():
    with open('/sys/class/net/lo/address') as f:
        hw_addr = f.read().splitlines()[0]
        if (hw_addr == '00:00:00:00:00:00'):
            hw_addr = ':'.join(['%02x']*6) % (
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255)
                        )
    return hw_addr

def read_cfgfile(my_ip, my_port, cfgfile):
    my_addr = ":".join([my_ip, my_port])

    config = configparser.ConfigParser()
    config.read(cfgfile)
    nodes = list(config['Nodes'].values())

    my_id = 0
    for node in nodes:
        addr = node
        if (addr == my_addr):
            break
        my_id += 1

    nbr_of_servers = int(config['General']['n'])
    f = int(config['General']['f'])
    e = int(config['General']['e'])
    base_location = config['General']['storage_location']
    max_clients = int(config['General']['max_clients'])
    delta = int(config['General']['concurrent_clients'])
    queue_size = int(config['General']['queue_size'])
    gossip_freq = int(config['General']['gossip_freq'])
    chunks_size = int(config['General']['chunks_size'])
    return [my_id, nbr_of_servers, f, e, base_location, max_clients,
            delta, queue_size, gossip_freq, chunks_size, nodes]

def start(my_ip, my_port, my_id, nbr_of_servers, f, e, base_location, max_clients,
         delta, queue_size, gossip_freq, chunks_size, nodes):
    k = nbr_of_servers - 2*(f + e)
    if(k < 1):
        raise Exception("Coded elements less than 1")
    quorum_size = math.ceil((nbr_of_servers + k + 2*e)/2)

    uid =  get_uid()
    server = Server(uid, quorum_size, max_clients, delta, queue_size,
        nbr_of_servers, storage_location="{}server{}/".format(base_location,
my_id), gossip_freq=gossip_freq)
    p = QuorumRecv(server)
    g = Gossip(server)

    loop = asyncio.get_event_loop()
    tag_tuple = server.get_tag_tuple()
    cntr = server.get_counter()
    # gossip_obj = GossipMessage(tag_tuple, cntr)
    # m = gossip_obj.get_bytes()
    m=None

    node_index = 0
    for node in nodes:
        ip, port = node.split(':')
        if node_index is not my_id:
            c = SenderChannel(node_index, uid, 1, g, ip, port, init_tx=m, chunks_size=chunks_size)
            loop.create_task(c.start())
        node_index += 1
    s = ServerRecvChannel(uid, p, g, my_port, my_ip, chunks_size=chunks_size)
    loop.create_task(s.tcp_listen())
    loop.create_task(s.udp_listen())

    loop.run_forever()
    loop.close()

def main(my_ip, my_port, cfgfile):
    parameters = read_cfgfile(my_ip, my_port, cfgfile)
    start(my_ip, my_port, *parameters)

if __name__ == '__main__':
    my_port = sys.argv[1]
    my_ip = sys.argv[2]
    cfgfile = sys.argv[3]
    main(my_ip, my_port, cfgfile)
