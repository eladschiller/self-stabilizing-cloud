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
import socket
'''
recvfrom and sendto are taken from:
https://github.com/crazyguitar/pysheeet/blob/master/docs/notes/python-asyncio.rst#simple-asyncio-udp-echo-server
under MIT license
'''
class UdpSender:
    """ Class to send and receive UDP messages with Asyncio """

    def __init__(self, loop, ip='', port=0):
        self.loop = loop
        sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, 0)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.setblocking(False)
        self.sock = sock
        self.sock.bind((ip, port))

    def recvfrom(self, n_bytes, fut=None, registed=False):
        fd = self.sock.fileno()
        if fut is None:
            fut = self.loop.create_future()
        if registed:
            self.loop.remove_reader(fd)

        try:
            data, addr = self.sock.recvfrom(n_bytes)
        except (BlockingIOError, InterruptedError):
            self.loop.add_reader(fd, self.recvfrom, n_bytes, fut, True)
        else:
            fut.set_result((data, addr))
        return fut

    def sendto(self, data, addr, fut=None, registed=False):
        fd = self.sock.fileno()
        if fut is None:
            fut = self.loop.create_future()
        if registed:
            self.loop.remove_writer(fd)
        if not data:
            return

        try:
            n = self.sock.sendto(data, addr)
        except (BlockingIOError, InterruptedError):
            self.loop.add_writer(fd, self.sendto, data, addr, fut, True)
        else:
            fut.set_result(n)
        return fut
