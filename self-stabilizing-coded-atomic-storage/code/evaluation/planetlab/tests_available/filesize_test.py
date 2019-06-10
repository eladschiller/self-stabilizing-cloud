#!/bin/python3.6
# -*- coding: utf-8 -*-
#
# License: MIT
#
############################     FILESISE TEST     ############################

import sys
import os
import pathlib
import configparser
from time import sleep
from time import time
from client import Client

def main(rounds, config, step):
  msg_sizes = ["1 K", "32 K", "128 K", "512 K", "1 M", "2 M", "4 M", "8 M"]
  delay = 2
  uid = os.uname().nodename
  pid = str(os.getpid())
  home=pathlib.Path.home()
  res = str(home) + "/results/"
  scenario = step.split('/')[-1]
  outliers = 1
  if rounds - 2*outliers < 1:
    print("Can't have more outliers than rounds!", file=sys.stderr)
    rounds = 2*outliers + 1
  results = configparser.ConfigParser()
  c = Client(config)
  print(f"Running filesize test at {uid}")
  for msg_size in msg_sizes:
    msg = os.urandom(to_bytes(*msg_size.split()))
    res_file = res + scenario + '_' + msg_size + '_' + '_' + uid + '.' + pid + '.log'
    results['Meta'] = {}
    results['Meta']['start_time'] = str(time())
    results['Meta']['uid'] = str(uid)
    results['Meta']['test'] = 'file size'
    results['Meta']['rounds'] = str(rounds)
    results['Meta']['outliers'] = str(2*outliers)
    results['Meta']['file_size'] = str(msg_size)
    results['Times_write'] = {}
    results['Times_read'] = {}
    results['Average_write'] = {}
    results['Average_read'] = {}
    for r in range(rounds):
      sleep(delay)
      # DO WORK HERE
      time_in_write = time()
      c.write(msg)
      time_out_write = time() - time_in_write
      time_in_read = time()
      c.read()
      time_out_read = time() - time_in_read
      results['Times_write'][f"run{r}"] = str(time_out_write)
      results['Times_read'][f"run{r}"] = str(time_out_read)
    # Writer
    times_write = sorted([ float(v) for v in results['Times_write'].values() ])
    no_outliers_write = times_write[outliers:-outliers]
    results['Average_write']['average'] = str(sum(times_write) / (rounds))
    results['Average_write']['average_no_outliers'] = str(sum(no_outliers_write) / (rounds - 2*outliers))
    # Reader
    times_read = sorted([ float(v) for v in results['Times_read'].values() ])
    no_outliers_read = times_read[outliers:-outliers]
    results['Average_read']['average'] = str(sum(times_read) / (rounds))
    results['Average_read']['average_no_outliers'] = str(sum(no_outliers_read) / (rounds - 2*outliers))

    pathlib.Path(res).mkdir(exist_ok=True, parents=True)
    try:
      with open(res_file, 'a') as f:
        results.write(f)
      print(f"Write test done!")
    except OSError as e:
      print(f"Error writing to file {res_file}: {e}", file=sys.stderr)

def to_bytes(s, prefix='b'):
  size = int(s)
  if (prefix == 'G'):
    return size*(2**30)
  elif (prefix == 'M'):
    return size*(2**20)
  elif (prefix == 'K'):
    return size*(2**10)
  else:
    return size

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
