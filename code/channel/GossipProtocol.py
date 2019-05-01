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
from .packHelper import PackHelper
from ast import literal_eval

class GossipMessage:
    """ Class that represents a gossip message """

    pack_helper = PackHelper()
    
    def __init__(self, tag_tuple, cntrs, prp, msg_all, echo):
        self.tag_tuple = str(tag_tuple).encode() if type(tag_tuple) != bytes else tag_tuple
        self.cntrs = str(cntrs).encode() if type(cntrs) != bytes else cntrs
        self.prp = str(prp).encode() if type(prp) != bytes else prp
        self.msg_all = str(msg_all).encode() if type(msg_all) != bytes else msg_all
        self.echo = str(echo).encode() if type(echo) != bytes else echo

    def set_message(msg):
        res_tuple = GossipMessage.pack_helper.unpack(msg)
        ctrl_data = res_tuple[0]
        tag_tuple = ctrl_data[0]
        cntrs = ctrl_data[1]
        prp = ctrl_data[2]
        msg_all = ctrl_data[3]
        echo = ctrl_data[4]
        return [tag_tuple, cntrs, prp, msg_all, echo]

    def get_bytes(self):
        msg = GossipMessage. \
              pack_helper.pack(self.tag_tuple, self.cntrs, self.prp, self.msg_all, self.echo)
        return msg

    def get_tag_tuple(self):
        return literal_eval(self.tag_tuple.decode())

    def get_cntrs(self):
        return literal_eval(self.cntrs.decode()) if self.cntrs else self.cntrs

    def get_prp(self):
        return literal_eval(self.prp.decode())

    def get_all(self):
        return literal_eval(self.msg_all.decode())

    def get_echo(self):
        return literal_eval(self.echo.decode())
