#!/bin/python3

import sys
import zmq

addr = sys.argv[1]

c = zmq.Context()
s = c.socket(zmq.ROUTER)
s.bind("tcp://%s:5550" % addr)
