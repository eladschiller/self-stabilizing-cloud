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
import re
from pathlib import Path

def main(rel_path):
    res_path = os.getcwd() + '/' + rel_path + '/'
    res_file = res_path + "summary.txt"

    results = configparser.ConfigParser()
    files = os.listdir(res_path)
    for i in ['reader', 'writer']:
      res_file = res_path + i + "_summary.txt"
      regex = re.compile('.*' + i + '.*log')
      filtered_files = [x for x in files if regex.match(x)]
      avg = []
      avg_no_outliers = []
      if filtered_files:
          for f in filtered_files:
              results.read(res_path + f)
              avg.append(float(results['Average']['average']))
              avg_no_outliers.append(float(results['Average']['average_no_outliers']))
              test = results['Meta']['test']
              rounds = results['Meta']['rounds']
              outliers = results['Meta']['outliers']
      else:
          print("No results file found!", file=sys.stderr)
          sys.exit()
      summary = sum(avg)/len(avg)
      summary_no_outliers = sum(avg_no_outliers)/len(avg_no_outliers)
      result = f"Average from {len(avg)} {test} clients during {rounds} rounds:\nAll: {summary}\nRemoved {outliers} outliers: {summary_no_outliers}"
      with open(res_file, 'a') as f:
          f.write(result)
      print(f"Created summary file {res_file}")

if __name__ == '__main__':
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        raise Exception("Missing argument: relative path")
    main(path)
