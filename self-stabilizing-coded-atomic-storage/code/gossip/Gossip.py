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

from channel.GossipProtocol import GossipMessage

class Gossip:

    def __init__(self, server):
        self.server = server

    async def departure(self, uid, msg_data):
        (reset, data) = await self.server.gossip_departure(uid)
        return (data, not self.use_tcp(data))

    async def arrival(self, uid, payload):
        if __debug__:
            print("Gossip CALLBACK RECV")
        if payload:
            msg_list = GossipMessage.set_message(payload)
            msg_data = GossipMessage(*msg_list)
            if __debug__:
                print(msg_data.get_tag_tuple())
            await self.server.gossip_arrival(uid, *msg_data.get_tag_tuple(),
msg_data.get_prp(), msg_data.get_all(), msg_data.get_echo(), msg_data.get_cntrs())
        else:
            if __debug__:
                print("Got empty message")

    def use_tcp(self, tx):
        if not tx:
            return False
        if (len(tx) > 512):
           return True
        else:
            return False
