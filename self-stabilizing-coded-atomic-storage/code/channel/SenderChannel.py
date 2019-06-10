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
import socket
import io
from .UdpSender import UdpSender

class SenderChannel:
    """ Creates an instance of a sender channel"""

    def __init__(self, sid, uid, channel_type, callback_obj,
                 ip, port, timeout = 2, init_tx = None, chunks_size = 1024):
        """
        Define all parameters that is specific to this channel
        """
        self.sid = sid
        self.uid = uid.encode()
        self.ch_type = channel_type
        self.cb_obj = callback_obj
        self.ip = ip
        self.port = int(port)
        self.udp_timeout = timeout
        self.tcp_timeout = timeout*5
        self.tx = init_tx
        self.chunks_size = chunks_size
        self.cap = 2**31

        self.loop = asyncio.get_event_loop()
        self.udp = True
        self.token_size = 2*struct.calcsize("i")+struct.calcsize("17s")
        self.addr  = (ip, int(port))
        self.udp_sock = UdpSender(self.loop)
        self.tc_sock = None

    async def receive(self, token):
        """
        Waits for data on either tcp or udp port to be received and then return the data.
        If no data is received in self.timeout seconds, assume msg is lost and
resend it.
        """
        while True:
            try:
                if self.udp:
                    res, addr = await asyncio.wait_for(self.udp_sock.recvfrom(self.chunks_size), self.udp_timeout)
                else:
                    res = await self.tcp_recv()
                token = res[:self.token_size]
                msg_type, msg_cntr, sender = struct.unpack("ii17s", token)
                msg_data = res[self.token_size:]
                break
            except Exception as e:
                msg = token+self.tx if self.tx else token
                if self.udp:
                    if __debug__:
                        print("Sending udp to %s" % str(self.addr))
                    await self.udp_sock.sendto(msg, self.addr)
                else:
                    await self.tcp_send(msg)
                if __debug__:
                    print("TIMEOUT: no response within {}s".format(self.udp_timeout))
        return (sender, msg_type, msg_cntr, msg_data)

    async def start(self):
        """
        main loop for a sender channel. Receive data and if it is a token
arrival construct a new message and send it.
        """
        counter = 1
        token = struct.pack("ii17s", self.ch_type, counter, self.uid)
        await asyncio.sleep(2)
        await self.udp_sock.sendto(token, self.addr)
        while True:
            token = struct.pack("ii17s", self.ch_type, counter, self.uid)
            sender, msg_type, msg_cntr, msg_data = await self.receive(token)
            if __debug__:
                print("Token arrival: cntr is {}".format(msg_cntr))
            if(msg_cntr >= counter):
                if self.ch_type:
                    self.tx, self.udp = await self.cb_obj.departure(sender, msg_data)
                else:
                    self.tx, self.udp = await self.cb_obj.departure(self.sid, msg_data)
                counter = (msg_cntr+1) % self.cap
                token = struct.pack("ii17s", self.ch_type, counter, self.uid)
                msg = token+self.tx if self.tx else token
                if self.udp:
                    await self.udp_sock.sendto(msg, self.addr)
                else:
                    await self.tcp_send(msg)

    async def tcp_connect(self):
        """
        Create a new tcp socket and wait until there is a connection
        """
        if self.tc_sock:
            self.tc_sock.close()
        self.tc_sock = socket.socket()
        self.tc_sock.setblocking(False)
        while True:
            try:
                await self.loop.sock_connect(self.tc_sock, (self.ip, self.port))
            except OSError as e:
                if __debug__:
                    print("Trying to connect to ({}, {})".format(self.ip, self.port))
                await asyncio.sleep(1)
            else:
                break
    
    async def tcp_recv(self):
        """
        Read a stream of tcp messages until the server closes the socket
        """
        msg = b''
        while True:
            res_part = await asyncio.wait_for(self.loop.sock_recv(self.tc_sock, self.chunks_size), self.tcp_timeout)
            if not res_part:
                break
            else:
                msg += res_part
        return msg
 
    async def tcp_send(self, msg):
        """
        Send tcp stream in chunks defined by chunk_size
        """
        await self.tcp_connect()
        msg_size = struct.pack("i", len(msg))
        response_stream = io.BytesIO(msg_size+msg)
        stream = True
        while stream:
            stream = response_stream.read(self.chunks_size)
            await self.loop.sock_sendall(self.tc_sock, stream)
