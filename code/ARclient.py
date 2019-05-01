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
from atomic_register.QuorumSendAR import QuorumSend
from channel.ppProtocol import PingPongMessage

class Client:

    def __init__(self, cfgfile):

        self.hw_addr = self.get_hwaddr()
        self.uid = (-1, self.hw_addr)
        self.loop = asyncio.get_event_loop()
        config = configparser.ConfigParser()
        config.read(cfgfile)
        nbr_of_servers = int(config['General']['n'])
        self.majority = math.ceil((nbr_of_servers+1)/2)
        quorum_size = self.majority
        self.p = QuorumSend(quorum_size, PingPongMessage)
        t = Thread(target=self.start_event_loop, args=(self.loop, config['Nodes']))
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

    def start_event_loop(self, loop, nodes):
        asyncio.set_event_loop(loop)
        i = 0
        for node in nodes:
            ip, port = nodes[node].split(':')
            c = SenderChannel(i, self.hw_addr, 0, self.p, ip, port)
            asyncio.ensure_future(c.start())
            print("Create channel to {}:{}".format(ip, port))
            i = i+1
        loop.run_forever()

    def qrmAccess(self, msg, opt_size=None):
        fut = asyncio.run_coroutine_threadsafe(self.p.phaseInit(msg, opt_size), self.loop)
        return fut.result()

    def write(self, msg):
        res = self.qrmAccess((None, None, 'qry', 'write'))
        try:
          max_tag = max([x.get_tag() for x in res])
        except Exception as e:
          print(res)
        new_int = int(max_tag[0])+1
        new_tag = (new_int, self.uid)
        self.qrmAccess((new_tag, msg, 'write', 'write'))

    def read(self):
        res = self.qrmAccess((None, None, 'qry', 'read'))
        max_tag = max([x.get_tag() for x in res])
        max_rec = [x for x in res if x.get_tag() == max_tag][0]
        self.qrmAccess((max_tag, max_rec.get_data(), 'inform', 'read'))
        return max_rec.get_data()
