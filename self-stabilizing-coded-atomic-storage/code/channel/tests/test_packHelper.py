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
from ..packHelper import PackHelper

class TestPackHelperMethods(unittest.TestCase):

    def test_pack(self):
        tag = b'tag'
        label = b'label'
        uid = b'uid'
        payload = b'payload'
        pack_helper = PackHelper()

        actual = pack_helper.pack(tag, label, uid)
        expected = struct.pack("i6s3s5s3s", 6, b'3s5s3s',
                               tag, label, uid)
        self.assertEqual(expected, actual)

        actual = pack_helper.pack(tag, label, uid,
                                  payload=payload)
        expected += payload
        self.assertEqual(expected, actual)
        
    def test_unpack(self):
        tag = b'tag'
        label = b'label'
        uid = b'uid'
        payload = b'payload'
        pack_helper = PackHelper()

        actual = pack_helper.unpack(pack_helper.pack(tag, label, uid))
        expected = ([tag, label, uid], None)
        self.assertEqual(expected, actual)

        actual = pack_helper.unpack(pack_helper.pack(tag, label, uid,
                                                     payload=payload))
        expected = ([tag, label, uid], payload)
        self.assertEqual(expected, actual)

if __name__ == '__main__':
    unittest.main()
