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

import sys
import zmq
import time
import signal
import pathlib
import os
import hashlib
from datetime import datetime
from ast import literal_eval

class server:

  def __init__(self, myaddr, addrs, verbose=False):
    pathlib.Path('data').mkdir(exist_ok=True)
    self.verbose = verbose
    self.state = {(0,0):(None,"fin")} 
    context = zmq.Context()
    self.socket = context.socket(zmq.ROUTER)
    myip = myaddr.split(':')[0]
    port = myaddr.split(':')[1]
    self.identity = "Server-{0}:{1}".format(myip,port)
    self.socket.identity = self.identity.encode()
    print(myaddr)
    self.socket.bind("tcp://*:%s" % port)
    self.socketid = {}
    self.sockets = []
    print("Running server on %s" % port)
    for addr in addrs:
      if ( addr != myaddr ):
        socket = context.socket(zmq.DEALER)
        self.socketid[socket] = "Server-%s" % addr
        socket.connect("tcp://%s" % addr);
        self.vprint("Server: Connecting to server with address {}".format(addr))
        self.sockets.append(socket)

  def vprint(self, *args, **kwargs):
    if self.verbose:
      print(*args, **kwargs)

  def receive(self, socket):
    msg = socket.recv_multipart()
    return_msg = msg
    msg = msg[2:]
    # msg = [b.decode() for b in m]
    self.vprint("[{me}] Received message {msg}".format(me=self.identity, msg=msg))
    self.log(msg, "r", socket.identity, msg[3])
    return return_msg

  def send(self, socket, msg, dest=None):
    socket.send_multipart(msg)
    # msg = str([msg_type, str(payload), msg_id])
    self.log(msg[1:], "s", socket.identity, dest)

  def start(self):
    while True:
      try:
        msg = self.receive(self.socket)
        returnid = msg[:1]
        msg = msg[2:]
        reply = self.make_reply(msg)
        if reply:
          reply = returnid + reply
          self.send(self.socket, reply, msg[3])
          self.vprint("[{me}] Sent {msg}".format(me=self.identity, msg=reply))
          if (reply[1] == b"WFINALIZE" or reply[1] == b"RFINALIZE"):
              msg_tag = self.unpackPayload(msg[1].decode())[0]
              self.sendGossip(msg_tag)
      except KeyboardInterrupt:
        print("Ctrl-C")
        socket.close()
        context.term()

  def make_reply(self, msg):
    msg_type = msg[0].decode()

    if (msg_type == "GOSSIP"):
      msg_tag = self.unpackPayload(msg[1].decode())
      self.recvGossip(msg_tag)
      return None

    msg_tag = self.unpackPayload(msg[1].decode())[0]
    msg_id = msg[2].decode()
    if (msg_type == "QUERY"):
      reply = ["QUERY", str(self.maxPhase()), msg_id]
    elif (msg_type == "PREWRITE"):
      content = msg[-1]
      reply = ["ACK", self.preWrite(msg_tag, content), msg_id]
    elif (msg_type == "RFINALIZE"):
      payload = self.finalizeReader(msg_tag)
      reply = ["RFINALIZE", payload[0], msg_id]
    elif (msg_type == "WFINALIZE"):
      reply = ["WFINALIZE", self.finalizeWriter(msg_tag), msg_id]

    encrep = [str(e).encode() for e in reply]
    if (msg_type == "RFINALIZE" and payload[1]):
      encrep = encrep+[payload[1]]
    return encrep

  def recvGossip(self, tag):
    if (tag in list(self.state.keys())):
      if(self.state[tag][0] == None):
        self.state[tag] = (None, 'fin')
    else:
        self.state[tag] = (None, 'fin')

  def sendGossip(self, tag):
    msg_id = str(self.hash(tag, datetime.now()))
    msg = ["GOSSIP", str(tag), msg_id, self.identity]
    bmsg = [e.encode() for e in [""]+msg]
    for s in self.sockets:
        s.send_multipart(bmsg)
        self.log(bmsg[1:], "s", self.socket.identity, self.socketid[s].encode())

  def finalizeReader(self, tag):
    if (tag in list(self.state.keys())):
      if(self.state[tag][0]):
        f = open(self.state[tag][0], 'rb+')
        element = f.read()
      else:
        element = None
      payload = (tag, element)
    else:
      self.state[tag] = (None, 'fin')
      payload = (tag, None)
    return payload

  def finalizeWriter(self, tag):
    if (tag in list(self.state.keys())):
      element = self.state[tag]
      self.state[tag] = (element[0], 'fin')
      self.vprint("[{me}] update: tag={tag}, msg={msg}".format(
        me=self.identity, tag=tag, msg=self.state[tag]))
    else:
      self.state[tag] = (None, 'fin')
    # self.cleanUpDict(tag)
    return "WFINALIZE"

  def preWrite(self, tag, content):
    if (tag not in list(self.state.keys())):
      filename = "data/{0}_{1}.txt".format(self.identity,str(tag))
      f = open(filename, 'wb+')
      f.write(content)
      self.state[tag] = (filename, 'pre')
      self.vprint("[{me}] add: tag={tag}, msg={msg}".format(
        me=self.identity, tag=tag, msg=self.state[tag]))
    return "PREWRITE"

  def cleanUpDict(self, newTag):
      for tag in list(self.state.keys()):
          if tag < newTag:
              self.vprint("[{me}] removing {tag}. New tag is {new}".format(
                me=self.identity, tag=tag, new=newTag))
              data = self.state[tag][0]
              del self.state[tag]
              if data:
                os.remove(data)

  #Return the largest tag value with 'fin'
  def maxPhase(self):
    maxtag = max(tag for tag, v in list(self.state.items()) if v[1] == 'fin')
    return maxtag

  #Create a list of message payload
  def unpackPayload(self, payload):
    payloadList = literal_eval(payload)
    return payloadList

  # returns the string which is the hexdigest of args using hash funciton hf
  def hash(self, *args):
    h = hashlib.new('sha256')
    for arg in args:
      h.update(str(arg).encode('ascii'))
    return h.hexdigest()

  def log(self, msg, mode, sock_id, dst):
    if self.verbose:
      mode = str(mode)
      dst = dst.decode()
      time = str(datetime.now())
      msg = str([e.decode() for e in msg[:4]])
      sock_id = sock_id.decode()
      log_str = "%26s %100s %2s %15s %15s" % (time, msg, mode, sock_id, dst)
      log_dir = 'log'
      log_file = log_dir + '/%s.log' % (self.identity)
      try:
        with open(log_file, 'a') as f:
          f.write(log_str + '\n')
      except OSError as e:
        print("Could not open logfile: {}".format(e), file=sys.stderr)

  def getval(self, msg):
    return literal_eval(msg[1])[0]

  def gettag(self, msg):
    return literal_eval(msg[1])[1]
