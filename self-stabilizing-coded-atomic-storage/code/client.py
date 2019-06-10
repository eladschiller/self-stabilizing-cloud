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
import configparser
import time
import threading
import math
import struct
import random as r
from threading import Thread
from channel.SenderChannel import SenderChannel
from quorum.QuorumSend import QuorumSend
from channel.ppProtocol import PingPongMessage
from pyeclib.ec_iface import ECDriver

class Client:

    def __init__(self, cfgfile):

        self.hw_addr = self.get_hwaddr()
        self.uid = (-1, self.hw_addr)
        self.loop = asyncio.get_event_loop()
        config = configparser.ConfigParser()
        config.read(cfgfile)
        nbr_of_servers = int(config['General']['n'])
        chunks_size = int(config['General']['chunks_size'])
        f = int(config['General']['f'])
        e = int(config['General']['e'])
        k = nbr_of_servers - 2*(f + e)
        if (k < 1):
            raise Exception("Coded elements less than 1")
        if (nbr_of_servers+k > 32):
            raise Exception("[liberasurecode] Total number of fragments (k + m) must be at most 32")
        quorum_size = math.ceil((nbr_of_servers + k + 2*e)/2)
        self.majority = math.ceil((nbr_of_servers+1)/2)
        self.ec_driver = ECDriver(k=k, m=nbr_of_servers, ec_type='liberasurecode_rs_vand')
        self.p = QuorumSend(quorum_size, PingPongMessage)
        t = Thread(target=self.start_event_loop, args=(self.loop, config['Nodes'], chunks_size))
        t.daemon = True
        t.start()

    def get_hwaddr(self):
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

    def start_event_loop(self, loop, nodes, chunks_size):
        asyncio.set_event_loop(loop)
        i = 0
        for node in nodes:
            ip, port = nodes[node].split(':')
            c = SenderChannel(i, self.hw_addr, 0, self.p, ip, port, chunks_size=chunks_size)
            asyncio.ensure_future(c.start())
            print("Create channel to {}:{}".format(ip, port))
            i = i+1
        loop.run_forever()

    def qrmAccess(self, msg, opt_size=None):
        fut = asyncio.run_coroutine_threadsafe(self.p.phaseInit(msg, opt_size), self.loop)
        return fut.result()

    def write(self, msg):
        elements = self.ec_driver.encode(msg)
        res = self.qrmAccess((None, None, 'qry', 'write'))
        max_tag = max([x.get_tag() for x in res])
        new_int = int(max_tag[0])+1
        new_tag = (new_int, self.uid)
        self.qrmAccess((new_tag, elements, 'pre', 'write'))
        self.qrmAccess((new_tag, None, 'fin', 'write'))
        self.qrmAccess((new_tag, None, 'FIN', 'write'))

    def read(self):
        res = self.qrmAccess((None, None, 'qry', 'read'))
        max_tag = max([x.get_tag() for x in res])
        res = self.qrmAccess((max_tag, None, 'fin', 'read'))
        elements = [x.get_data() for x in res if x.get_data()]
        try:
            decoded_msg = self.ec_driver.decode(elements)
        except:
            decoded_msg = None
        return decoded_msg

    def check_uid(self):
        res = self.qrmAccess((None, None, 'cntrQry', None), opt_size=self.majority)
        max_cntr = max([struct.unpack("i", x.get_data()) for x in res])[0]
        if (max_cntr != self.uid[0]):
            new_inc_nbr = max(max_cntr, self.uid[0])+1
            new_cntr = struct.pack("i", new_inc_nbr)
            self.qrmAccess((None, new_cntr, 'incCntr', None), opt_size=self.majority)
            self.uid = (new_inc_nbr, self.hw_addr)
