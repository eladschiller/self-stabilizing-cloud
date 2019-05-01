#!/bin/python3.5
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

import zmq
import time
import signal
import math
import pathlib
from .cas import cas
from .server import server
from random import randint

def start_reader(ports, uid, verbose=False):
  ports = ports.split()
  s = cas(ports, uid, verbose)
  pathlib.Path('data').mkdir(exist_ok=True)
  filename = "data/reader-{0}.txt".format(str(uid))
  time.sleep(1)
  while True:
    try:
      val = s.read()
      if(val):
        f = open(filename, 'wb+')
        f.write(val)
      # print("READER: Latest value %s" % val)
      time.sleep(3)
    except KeyboardInterrupt:
      print("Ctrl-C")
      context.term()

def start_writer(ports, uid, datafile=None, verbose=False):
  ports = ports.split()
  s = cas(ports, uid, verbose)
  time.sleep(1)
  while True:
    try:
      if datafile:
        val = open(datafile, 'rb').read()
      else:
        val = b"test%d" % randint(0,1000)
      # print("val=%s" % val)
      s.write(val)
      time.sleep(1)
    except KeyboardInterrupt:
      print("Ctrl-C")
      context.term()

def start_server(port, addr, verbose=False):
  s = server(port, addr, verbose)
  s.start()
