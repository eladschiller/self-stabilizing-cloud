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
from channel.ppProtocol import PingPongMessage

class QuorumRecv:

    def __init__(self, server):
        self.server = server

    async def arrival(self, sender, payload):
        msg_list = PingPongMessage.set_message(payload)
        msg_data = PingPongMessage(*msg_list)
        if msg_data:
            tag = msg_data.get_tag()
            data = msg_data.get_data()
            label = msg_data.get_label()
            mode = msg_data.get_mode()
            if (label == 'qry'):
                if (mode == 'read'):
                    res = self.server.read_query()
                    if not res:
                        return None
                else:
                    res = self.server.write_query()
                    if not res:
                        return None
            elif (label == 'pre'):
                res = await self.server.pre_write(tag, data)
            elif (label == 'fin' or label == 'FIN'):
                if (mode == 'read'):
                    res = await self.server.read_finalize(tag, label)
                else:
                    res = await self.server.write_finalize(tag, label)
            elif (label == 'cntrQry'):
                res = self.server.counter_query(sender)
                if not res:
                    return None
            elif (label == 'incCntr'):
                nbr = struct.unpack("i", data)[0]
                res = self.server.set_counter(sender, nbr)
            else:
                return None
            new_msg = PingPongMessage(*res, mode, req_tag=msg_data.get_tag())
            return new_msg.get_bytes()
        else:
            if __debug__:
                print("Got empty message")
            return None
