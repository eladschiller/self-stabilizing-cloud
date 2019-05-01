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
from .ppProtocol import PingPongMessage
from .GossipProtocol import GossipMessage
from .UdpSender import UdpSender

class ServerRecvChannel:
    """ Creates a server recv channel for pingpong and gossip"""

    def __init__(self, uid, callback_obj, callback_obj_gossip, port, ip,
                 chunks_size=1024):
        """
        Initialize callbacks, parameters and create tcp/udp sockets
        """

        self.uid = uid.encode()
        self.cb_obj = callback_obj
        self.cb_obj_gossip = callback_obj_gossip
        self.port = port
        self.chunks_size = chunks_size

        self.loop = asyncio.get_event_loop()
        self.udp_sock = UdpSender(self.loop, ip, int(port))
        self.token_size = 2*struct.calcsize("i")+struct.calcsize("17s")
        self.tokens = {}

        self.tc_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.tc_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        self.tc_sock.setblocking(False)
        self.tc_sock.bind((ip, int(port)))
        # Default backlog size in Linux is 128 (see /proc/sys/net/core/somaxconn)
        self.tc_sock.listen()

    async def tcp_listen(self):
        """
        Wait for tcp connections to arrive
        """
        print("Listening for tcp connections")
        while True:
            conn, addr = await self.loop.sock_accept(self.tc_sock)
            if __debug__:
                print("{} got tcp connection from {}".format(self.port, addr))
            asyncio.ensure_future(self.tcp_response(conn))

    async def udp_listen(self):
        """
        Wait until udp message arrives.
        """
        print("Listening for udp connections")
        while True:
            data, addr = await self.udp_sock.recvfrom(self.chunks_size)
            if __debug__:
                print("{} got udp request from {}".format(self.port, addr))
            asyncio.ensure_future(self.udp_response(data, addr))

    async def udp_response(self, data, addr):
        """
        Create udp response and send it.
        """
        response = await self.check_msg(data)
        if response:
            await self.udp_sock.sendto(response, addr)

    async def tcp_response(self, conn):
        """
        Receive tcp stream, create response and send it
        """
        int_size = struct.calcsize("i")
        recv_msg_size = await self.loop.sock_recv(conn, int_size)
        try:
            msg_size = struct.unpack("i", recv_msg_size)[0]
        except Exception as e:
            conn.close()
            return
        res = b''
        while (len(res) < msg_size):
            res += await self.loop.sock_recv(conn, self.chunks_size)
            await asyncio.sleep(0)
        response = await self.check_msg(res)
        response_stream = io.BytesIO(response)
        stream = True
        while stream:
            stream = response_stream.read(self.chunks_size)
            try:
                await self.loop.sock_sendall(conn, stream)
            except Exception as e:
                conn.close()
                return
        conn.close()
        if __debug__:
            print("Connection closed")
        
    async def check_msg(self, res):
        """
        Determine message type and create response message accordingly
        """
        token = res[:self.token_size]
        payload = res[self.token_size:]
        msg_type, msg_cntr, sender = struct.unpack("ii17s", token)
        
        if(sender not in self.tokens.keys()):
            if __debug__:
                print("Add new token")
            self.tokens[sender] = 0

        if(self.tokens[sender] != msg_cntr):
            self.tokens[sender] = msg_cntr
            token = struct.pack("ii17s", msg_type,self.tokens[sender], self.uid)
            if(msg_type == 0):
                if payload:
                    new_msg = await self.cb_obj.arrival(sender, payload)
                    if new_msg:
                        response = token+new_msg if new_msg else token
                    else:
                        response = token
                else:
                    response = token
            elif(msg_type == 1):
                await self.cb_obj_gossip.arrival(sender, payload)
                response = token
        else:
            if __debug__:
                print("NO TOKEN ARRIVAL")
            token = struct.pack("ii17s", msg_type,self.tokens[sender], self.uid)
            response = token

        return response
