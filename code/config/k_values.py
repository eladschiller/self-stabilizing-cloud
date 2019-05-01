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
import math

def main(nmax=20, latex=False):
  n = nmax
  fmax = math.ceil(n/2)
  k_table(nmax, fmax, latex=latex)
  return

def k_table(nmax, fmax, latex=False):
  print(f"%%%%% Matrix for k value  %%%%%")
  matrix=[]
  matrix.append([' '])
  for i, n in enumerate(range(3, nmax+1)):
    matrix.append([n])
  for f in range(fmax +1):
    matrix[0].append(f)
    for i, n in enumerate(range(3, nmax+1)):
      kmax = n-2*f
      if kmax > 0:
        matrix[i+1].append(kmax)
  print_table(matrix, nmax, fmax, latex)
  return

def print_table(matrix, nmax, fmax, latex):
  if latex:
    first = True
    for row in matrix:
      row_len = len(row)
      fmt = " {:>3} &"
      row_format = fmt * (row_len-1)
      for i in range(fmax - (row_len-1) + 1):
        row.append('')
        row_format = row_format + fmt
      row_format = row_format + " {:>3}\\\\"
      if first:
        row_format = row_format + "\\hline"
        first = False
      print(row_format.format( *row ))
  else:
    matrix.append(['n'])
    matrix[0].append('f')
    for row in matrix:
      row_format = "{:>2} " * len(row)
      print(row_format.format( *row ))

if __name__ == '__main__':
  program = sys.argv[0]
  latex = False
  nmax = 20
  if len(sys.argv) > 1:
    latex=True
  if len(sys.argv) > 2:
    nmax = int(sys.argv[2])
  try:
    main(nmax=nmax, latex=latex)
  except KeyboardInterrupt:
    sys.exit("\nInterrupted by ^C\n")

