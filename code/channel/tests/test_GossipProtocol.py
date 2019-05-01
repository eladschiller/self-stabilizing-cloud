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

import unittest
import struct
from channel.GossipProtocol import GossipMessage

class TestGossipMessage(unittest.TestCase):

    def test_gossip_message(self):
        tag_tuple = ((3, (2, None)), (2, (2, None)), (1, (2, None)))
        prp = (1, None)
        msg_all = False
        cntrs = None
        echo = (prp, msg_all)
        gossip_msg = GossipMessage(tag_tuple, cntrs, prp, msg_all, echo)
        self.assertEqual(tag_tuple, gossip_msg.get_tag_tuple())
        self.assertEqual(cntrs, gossip_msg.get_cntrs())
        self.assertEqual(prp, gossip_msg.get_prp())
        self.assertEqual(msg_all, gossip_msg.get_all())
        self.assertEqual(echo, gossip_msg.get_echo())

if __name__ == '__main__':
    unittest.main()
