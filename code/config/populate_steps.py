#!/bin/python
# -*- coding: utf-8 -*-
#
# License: MIT
# Author: Andreas Lindhé

import sys
import time
import itertools
import pathlib

def main(r, w, s):
  c_nodes="config/clients.txt"
  s_nodes="config/servers.txt"
  r = [int(e) for e in r]
  w = [int(e) for e in w]
  s = [int(e) for e in s]
  with open(s_nodes) as f:
    servers = f.read().splitlines()
  if len(r) == 1 and len(w) > 1:
    r = r[0]
    s = s[0]
    with open(c_nodes) as f:
      client_nodes = f.read().splitlines()
      for writer in w:
        clients = r + writer
        client_list = (client_nodes*clients)[:clients]
        rs = client_list[:r]
        ws = client_list[-writer:]
        ss = servers[:s]
        path = "config/tests/test-writers/" + "step" + str(writer) + '/'
        pathlib.Path(path).mkdir(exist_ok=True, parents=True)
        with open(path + "writers.txt", 'w') as f:
          for line in ws:
            f.write(line + '\n')
        with open(path + "readers.txt", 'w') as f:
          for line in rs:
            f.write(line + '\n')
        with open(path + "servers.txt", 'w') as f:
          for line in ss:
            f.write(line + '\n')
  elif len(w) == 1 and len(r) > 1:
    w = w[0]
    s = s[0]
    with open(c_nodes) as f:
      client_nodes = f.read().splitlines()
      for reader in r:
        clients = w + reader
        client_list = (client_nodes*clients)[:clients]
        rs = client_list[-reader:]
        ws = client_list[:w]
        ss = servers[:s]
        path = "config/tests/test-readers/" + "step" + str(reader) + '/'
        pathlib.Path(path).mkdir(exist_ok=True, parents=True)
        with open(path + "writers.txt", 'w') as f:
          for line in ws:
            f.write(line + '\n')
        with open(path + "readers.txt", 'w') as f:
          for line in rs:
            f.write(line + '\n')
        with open(path + "servers.txt", 'w') as f:
          for line in ss:
            f.write(line + '\n')
  else:
    w = w[0]
    r = r[0]
    for server in s:
      ss = (servers*server)[:server]
      path = "config/tests/test-servers/" + "step" + str(server) + '/'
      pathlib.Path(path).mkdir(exist_ok=True, parents=True)
      with open(path + "servers.txt", 'w') as f:
        for line in ss:
          f.write(line + '\n')
      with open(c_nodes) as f:
        client_nodes = f.read().splitlines()
        clients = w + r
        client_list = (client_nodes*clients)[:clients]
        rs = client_list[:r]
        ws = client_list[-w:]
        with open(path + "writers.txt", 'w') as f:
          for line in ws:
            f.write(line + '\n')
        with open(path + "readers.txt", 'w') as f:
          for line in rs:
            f.write(line + '\n')

if __name__ == '__main__':
  program = sys.argv[0]
  readers = sys.argv[1].split(',')
  writers = sys.argv[2].split(',')
  servers = sys.argv[3].split(',')
  try:
    main(readers, writers, servers)
  except KeyboardInterrupt:
    sys.exit("\nInterrupted by ^C\n")

