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

import sys
import os
import pathlib
import configparser
from time import sleep
from time import time
from client import Client

def main(rounds, config, step):
  msg_size = 256
  msg = os.urandom(msg_size)
  max_val = 2147483647
  delay = 2
  uid = os.uname().nodename
  pid = str(os.getpid())
  home=pathlib.Path.home()
  res = str(home) + "/results/"
  scenario = step.split('/')[-1]
  res_file = res + scenario + '_' + uid + '.' + pid + '.log'
  outliers = 1
  if rounds - 2*outliers < 1:
    print("Can't have more outliers than rounds!", file=sys.stderr)
    rounds = 2*outliers + 1
  results = configparser.ConfigParser()
  acc_time = 0
  # Results
  results['Meta'] = {}
  results['Meta']['start_time'] = str(time())
  results['Meta']['uid'] = str(uid)
  results['Meta']['test'] = 'reset'
  results['Meta']['rounds'] = str(rounds)
  results['Meta']['outliers'] = str(2*outliers)
  results['Meta']['file_size'] = str(msg_size)
  results['Times'] = {}
  results['Average'] = {}
  c = Client(config)
  max_tag = (max_val, (1, uid))
  print(f"Running reset test at {uid}")
  c.qrmAccess((None, None, 'qry', 'write'))
  for r in range(rounds):
    sleep(delay)
    # DO WORK HERE
    time_in = time()
    c.qrmAccess((max_tag, msg, 'pre', 'write'))
    c.qrmAccess((None, None, 'qry', 'write'))
    time_out = time() - time_in
    results['Times'][f"run{r}"] = str(time_out)
    print(f"Test {r+1}/{rounds} took {time_out} seconds.")
  times = sorted([ float(v) for v in results['Times'].values() ])
  no_outliers = times[outliers:-outliers]
  results['Average']['average'] = str(sum(times) / (rounds))
  results['Average']['average_no_outliers'] = str(sum(no_outliers) / (rounds - 2*outliers))
  pathlib.Path(res).mkdir(exist_ok=True, parents=True)
  try:
    with open(res_file, 'a') as f:
      results.write(f)
    print(f"Reset test done!")
  except OSError as e:
    print(f"Error writing to file {res_file}: {e}", file=sys.stderr)

if __name__ == '__main__':
  program = sys.argv[0]
  rounds = int(sys.argv[1]) if len(sys.argv) > 1 else 5
  home=str(pathlib.Path.home())
  config = sys.argv[2] if len(sys.argv) > 2 else f"{home}/casss/config/local.ini"
  step_name = sys.argv[3] if len(sys.argv) > 3 else "step0"
  try:
    main(rounds, config, step_name)
  except KeyboardInterrupt:
    sys.exit("\nInterrupted by ^C\n")
