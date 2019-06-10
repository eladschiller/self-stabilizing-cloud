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

import struct
import sys
import configparser
import math
import random as r
from cas.old.server import server


def read_cfgfile(my_port, my_ip, cfgfile):
    my_addr = ":".join([my_ip, my_port])

    config = configparser.ConfigParser()
    config.read(cfgfile)
    nodes = list(config['Nodes'].values())


    nbr_of_servers = int(config['General']['n'])
    f = int(config['General']['f'])
    e = int(config['General']['e'])
    base_location = config['General']['storage_location']
    max_clients = int(config['General']['max_clients'])
    delta = int(config['General']['concurrent_clients'])
    queue_size = int(config['General']['queue_size'])
    gossip_freq = int(config['General']['gossip_freq'])
    chunks_size = int(config['General']['chunks_size'])
    return [nbr_of_servers, f, e, base_location, max_clients,
            delta, queue_size, gossip_freq, chunks_size, nodes]

def start(my_port, my_ip, nbr_of_servers, f, e, base_location, max_clients,
         delta, queue_size, gossip_freq, chunks_size, nodes):

    #print(my_ip, my_port)
    my_addr = my_ip + ':' + my_port
    other_nodes = [node for node in nodes if my_addr != node]

    print(other_nodes)
    s = server(my_addr, other_nodes)
    s.start()


def main(my_port, my_ip, cfgfile):
    parameters = read_cfgfile(my_ip, my_port, cfgfile)
    start(my_port, my_ip, *parameters)

if __name__ == '__main__':
    my_port = sys.argv[1]
    my_ip = sys.argv[2]
    cfgfile = sys.argv[3]
    main(my_port, my_ip, cfgfile)
