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
    res_file = "test-filesize_summary.csv"

    results = configparser.ConfigParser()
    files = os.listdir(res_path)
    regex = re.compile('.*log')
    filtered_files = [x for x in files if regex.match(x)]
    for i in filtered_files:
      results.read(res_path + i)
      avg_w = float(results['Average_write']['average'])
      avg_r = float(results['Average_read']['average'])
      avg_no_outliers_w = float(results['Average_write']['average_no_outliers'])
      avg_no_outliers_r = float(results['Average_read']['average_no_outliers'])
      test = results['Meta']['test']
      rounds = results['Meta']['rounds']
      outliers = results['Meta']['outliers']
      filesize = to_KiB(*results['Meta']['file_size'].split(' '))
      result = f"{filesize}\t{avg_no_outliers_r}\t{avg_no_outliers_w}\n"
      print(result)
      with open(res_file, 'a') as f:
        f.write(result)
      print(f"Created summary file {res_file}")

def to_KiB(s, prefix='b'):
  size = int(s)
  if (prefix == 'G'):
    return size*(1024**2)
  elif (prefix == 'M'):
    return size*1024
  elif (prefix == 'K'):
    return size
  else:
    return size/1024

if __name__ == '__main__':
    if len(sys.argv) > 1:
        path = sys.argv[1]
    else:
        raise Exception("Missing argument: relative path")
    main(path)
