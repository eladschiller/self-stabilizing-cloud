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
from channel.GossipProtocol import GossipMessage
from channel.ppProtocol import PingPongMessage
from register.fileHelper import read_file, write_file


class Server:

  def __init__(self, uid, quorum, storage_location="./.storage/"):
    self.uid = uid
    # Quorum size:
    self.filename = str(uid) + ".elem"
    self.storage_location = storage_location
    self.tag = (0, None)
    self.quorum = quorum
    # Initialize with an empty register S

  async def read_query(self):
    data = await read_file(self.filename, path=self.storage_location)
    return (self.tag, data, 'qry')

  def write_query(self):
    return (self.tag, None, 'qry')

  async def write(self, t, w):
    if t > self.tag:
      self.tag = t 
      await write_file(w, self.filename, path=self.storage_location)
    return (t, None, 'write')

  async def inform(self, t, w):
    if t > self.tag:
      self.tag = t 
      await write_file(w, self.filename, path=self.storage_location)
    return (t, None, 'inform')
