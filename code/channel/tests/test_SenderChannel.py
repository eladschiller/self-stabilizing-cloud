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
import mock
import asyncio
import pytest
from unittest.mock import patch
from ..ppProtocol import PingPongMessage
from ..SenderChannel import SenderChannel
from .unittest_helper import run, mock_async_method

class CallbackObj:
    async def callback(self):
        return "test"

hw_addr = "00:00:00:00:00:00"
pingpong = SenderChannel(0, hw_addr, 0, CallbackObj(), "127.0.0.1", "5555")
token = (struct.pack("ii17s", 0, 1, hw_addr.encode()), "127.0.0.1")

class TestSenderChannel(unittest.TestCase):

    @patch.object(pingpong.udp_sock, 'recvfrom', new=mock_async_method(return_value=token))
    def test_receive_no_exception(self):
        actual = run(pingpong.receive(token))
        self.assertEqual((hw_addr.encode(), 0, 1, b''), actual)
        pingpong.udp_sock.sock.close()

    @patch.object(pingpong.udp_sock, 'recvfrom', new=mock_async_method(return_value=None))
    @patch.object(pingpong.udp_sock, 'sendto', side_effect=Exception('timeout'))
    def test_receive_with_exception(self, mock_send):
        with self.assertRaises(Exception):
            run(pingpong.receive(token))
        pingpong.udp_sock.sock.close()

if __name__ == '__main__':
    unittest.main()
