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
import math
import zmq
import configparser
import random as r
from datetime import datetime
import hashlib
from ast import literal_eval
from pyeclib.ec_iface import ECDriver

class cas:

  def __init__(self, cfgfile, verbose=False):
    uid = self.get_hwaddr()
    config = configparser.ConfigParser()
    config.read(cfgfile)
    N = int(config['General']['n'])
    F = int(config['General']['f'])
    addrs = list(config['Nodes'].values())
    self.verbose = verbose
    context = zmq.Context()
    self.k = N-2*F
    self.ec_driver = ECDriver(k=self.k, m=N, ec_type='liberasurecode_rs_vand')
    self.socketid = {}
    self.sockets = []
    self.uid = uid
    self.id = "Client-{}".format(uid)
    self.poller = zmq.Poller()
    self.quorum = int(math.ceil((N+self.k)/2))
    print("Starting client {}".format(uid))
    for addr in addrs:
      socket = context.socket(zmq.DEALER)
      self.socketid[socket] = "Server-%s" % addr
      self.poller.register(socket, zmq.POLLIN)
      socket.connect("tcp://%s" % addr);
      self.vprint("Connecting to server with address {}".format(addr))
      self.sockets.append(socket)

  def get_hwaddr(self):
    with open('/sys/class/net/lo/address') as f:
      hw_addr = f.read().splitlines()[0]
      if (hw_addr == '00:00:00:00:00:00'):
        hw_addr = ':'.join(['%02x']*6) % (
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255),
                        r.randint(0, 255)
                        )
    return hw_addr

  def vprint(self, *args, **kwargs):
    if self.verbose:
      print(*args, **kwargs)

  def createMessage(self, msg):
    elements = self.ec_driver.encode(msg)
    return elements

  def assembleMessage(self, elements):
    decoded_msg = self.ec_driver.decode(elements)
    return decoded_msg

  #Send different coded element to everyone
  def sendCodedElements(self, msg, tag, msg_id):
    elements = self.createMessage(msg)
    for i in range(len(self.sockets)):
      payload = (tag, 0, 'pre')
      msg = ["PREWRITE", str(payload), msg_id, str(self.uid)]
      bmsg = [e.encode() for e in [""]+msg] + [elements[i]]
      self.sockets[i].send_multipart(bmsg)
      # mon_msg = str(["PREWRITE", str(payload), msg_id])
      self.log(bmsg[1:], "s", self.socketid[self.sockets[i]])

  def send(self, msg_type, payload, msg_id):
    msg = [msg_type, str(payload), msg_id, str(self.uid)]
    bmsg = [e.encode() for e in [""]+msg]
    for s in self.sockets:
      s.send_multipart(bmsg)
      self.log(bmsg[1:], "s", self.socketid[s])

  def receive(self, socket, expected=None):
    msg = socket.recv_multipart()
    self.log(msg, "r", self.socketid[socket])
    if expected:
      if msg[2] == expected:
        return msg
      else:
        self.vprint("[{me}] Did not match message: {msg}".format(me=self.id, msg=msg))
        return None
    return msg
 
  def preWrite(self, msg, tag):
    msg_id = str(self.hash(tag, datetime.now()))
    self.sendCodedElements(msg, tag, msg_id)
    acks = 0
    while acks < self.quorum:
      reply = dict(self.poller.poll())
      while reply:
        socket, event = reply.popitem()
        message = self.receive(socket, expected=msg_id.encode())
        if message:
          self.vprint("[{me}] Received (pre-write) message: {msg}".format(
            me=self.id, msg=message[1].decode()))
          if message[0] == b"ACK":
            acks += 1

  def readerFinalize(self, tag):
    acks = 0
    listOfElements = []
    payload = (tag, None, 'fin')
    msg_id = str(self.hash(datetime.now(), self.uid))
    self.send("RFINALIZE", payload, msg_id)
    while acks < self.quorum:
      reply = dict(self.poller.poll())
      while reply:
        socket, event = reply.popitem()
        message = self.receive(socket, expected=msg_id.encode())
        if message:
          self.vprint("[{me}] Received (read-fin) message: {msg}".format(
            me=self.id, msg=message[1].decode()))
          msg_type = message[0]
          if(msg_type == b"RFINALIZE"):
            if(len(message) == 4):
              listOfElements.append(message[-1])
            acks += 1
    if (len(listOfElements) >= self.k):
      self.vprint("[{me}] Got enough elements to assemble message!".format(me=self.id))
      return self.assembleMessage(listOfElements)
    else:
      self.vprint("[{me}] Did not get enough elements to assemble message!".format(me=self.id))
      return None

  def query(self):
    maxtag = (0,0)
    acks = 0
    payload = (None, None, self.uid)
    msg_id = str(self.hash(datetime.now(), self.uid))
    self.send("QUERY", payload, msg_id)
    while acks < self.quorum:
      reply = dict(self.poller.poll())
      while reply:
        socket, event = reply.popitem()
        message = self.receive(socket, expected=msg_id.encode())
        self.vprint("[{me}] Received (query) message: {msg}".format(
          me=self.id, msg=message))
        if message:
          msg_type = message[0]
          if(msg_type == b"QUERY"):
            msg_maxtag = literal_eval(message[1].decode())
            self.vprint("[{me}] Received maxts: {m}".format(me=self.id, m=msg_maxtag))
            if(self.compareTags(msg_maxtag, maxtag)):
              maxtag = msg_maxtag
            acks += 1
    return maxtag

  def read(self):
    #QUERY PHASE
    ts = self.query()
    self.vprint("[{me}] maxts = {m}".format(me=self.id, m=ts))
    res = self.readerFinalize(ts)
    return res

  def write(self, message):
    #QUERY PHASE
    maxtag = self.query()
    newTag = (maxtag[0]+1, self.uid)

    #PRE-WRITE PHASE
    self.preWrite(message, newTag)

    #WRITER FINALIZE PHASE
    acks = 0
    payload = (newTag, None, 'fin')
    msg_id = str(self.hash(payload, datetime.now()))
    self.send("WFINALIZE", payload, msg_id)
    self.vprint("[{me}] sent: msg_id={mid}".format(
      me=self.id, mid=msg_id))
    while acks < self.quorum:
      reply = dict(self.poller.poll())
      while reply:
        socket, event = reply.popitem()
        message = self.receive(socket, expected=msg_id.encode())
        if message:
          msg_type = message[0]
          if(msg_type == b"WFINALIZE"):
            acks += 1

  #Return True if the first one is larger
  def compareTags(self, firstTuple, secondTuple):
    return firstTuple > secondTuple

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

  def log(self, msg, mode, dst):
    if self.verbose:
      uid = str(self.uid)
      mode = str(mode)
      time = str(datetime.now())
      msg = str([e.decode() for e in msg[:4]])
      log_str = "%26s %100s %2s %15s %15s" % (time, msg, mode, uid, dst)
      log_dir = 'log'
      log_file = log_dir + '/client_%s.log' % (uid)
      try:
        with open(log_file, 'a') as f:
          f.write(log_str + '\n')
      except OSError as e:
        print("Could not open logfile: {}".format(e), file=sys.stderr)
