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

class QuorumSend:
    def __init__(self, quorum_size, protocol):
        self.pongRx = {}
        self.protocol = protocol
        self.Q = quorum_size
        self.replies = quorum_size
        self.pingTx = None
        self.event = None
        self.aggregated = None

    async def phaseInit(self, m, opt_size=None):
        if opt_size:
            self.replies = opt_size
        else:
            self.replies = self.Q
        self.pongRx.clear()
        if (type(m[1]) == list):
            self.pingTx = []
            for i in range(len(m[1])):
                self.pingTx.append(self.protocol(m[0], m[1][i], *m[2:]))
        else:
            self.pingTx = self.protocol(*m)
        self.event = asyncio.Event()
        await self.event.wait()
        x = self.aggregated
        self.aggregated = None
        return x

    async def departure(self, server_id, payload):
        if __debug__:
            print("pingpong arrival! ")
        if (type(self.pingTx) == list):
            pingTx = self.pingTx[server_id]
        else:
            pingTx = self.pingTx

        if payload and (pingTx != None):
            msg_list = self.protocol.set_message(payload)
            msg = self.protocol(*msg_list)
            if(msg.get_req_tag() == pingTx.get_tag() and
               (msg.get_tag() == None or
               msg.get_label() == 'qry' or
               (msg.get_label() != 'qry' and
               (msg.get_tag() == msg.get_req_tag() and (
                msg.get_label() == pingTx.get_label())
              )))):
                self.pongRx[server_id] = msg
                if __debug__:
                    print("ADD to pongRx with size %s" % len(self.pongRx))
        elif not payload:
            self.pongRx.pop(server_id, None)

        if len(self.pongRx) >= self.replies:
            if __debug__:
                print("GOT ENOUGH elements")
            self.aggregated = list(self.pongRx.values())
            self.pongRx.clear()
            self.pingTx = None
            self.event.set()

        if (type(self.pingTx) == list):
            data = self.pingTx[server_id].get_bytes()
            return (data, not self.use_tcp(self.pingTx[server_id]))
        else:
            data = self.pingTx.get_bytes() if self.pingTx else None
            return (data, not self.use_tcp(self.pingTx))

    def use_tcp(self, tx):
        if not tx:
            return False
        if (len(tx.get_bytes()) > 512):
           return True
        elif ((tx.get_label() == 'fin' or tx.get_label() == 'FIN') and tx.get_mode() == 'read'):
            return True
        else:
            return False
