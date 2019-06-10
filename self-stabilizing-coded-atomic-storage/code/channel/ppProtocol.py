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

from .packHelper import PackHelper
from ast import literal_eval

class PingPongMessage:
    """ Class that represents a PingPong message """
    pack_helper = PackHelper()
    
    def __init__(self, tag, data, label, mode, req_tag=None):
        self.tag = str(tag).encode() if type(tag) != bytes and tag else tag
        self.data = data
        self.label = label.encode() if type(label) != bytes and label else label
        self.mode = str(mode).encode() if type(mode) != bytes and mode else mode
        self.req_tag = str(req_tag).encode() if type(req_tag) != bytes else req_tag

    def set_message(msg):
        res_tuple = PingPongMessage.pack_helper.unpack(msg)
        ctrl_data = res_tuple[0]
        data = res_tuple[1]
        tag = ctrl_data[0]
        label = ctrl_data[1]
        mode = ctrl_data[2]
        req_tag = ctrl_data[3]
        return [tag, data, label, mode, req_tag]

    def get_bytes(self):
        msg = PingPongMessage. \
              pack_helper.pack(self.tag, self.label, self.mode, self.req_tag, payload=self.data)
        return msg

    def get_tag(self):
        return literal_eval(self.tag.decode()) if self.tag else self.tag

    def get_data(self):
        return self.data

    def get_label(self):
        return self.label.decode() if self.label else self.label

    def get_req_tag(self):
        return literal_eval(self.req_tag.decode())

    def get_mode(self):
        """ Mode is 'write' or 'read' """
        return self.mode.decode() if self.mode else self.mode
