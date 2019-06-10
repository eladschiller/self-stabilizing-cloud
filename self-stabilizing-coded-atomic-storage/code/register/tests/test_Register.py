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

import unittest
import itertools
from register.record import Record
from register.register import Register
from channel.tests.unittest_helper import run

class TestRegister(unittest.TestCase):
  def setUp(self):
    max_clients = 10
    delta = 10
    self.s = Register(max_clients, delta)
    self.phases = ['pre', 'fin', 'FIN']
    pre_max = 20
    fin_max = 12
    FIN_max = 6
    self.pre_max = pre_max
    self.fin_max = fin_max
    self.FIN_max = FIN_max
    for i in range(2, FIN_max):
      x = ((i, 'dflt'), None, 'FIN')
      r = Record(*x)
      run(self.s.update_phase(*x))
    for i in range(FIN_max, fin_max):
      x = ((i, 'dflt'), None, 'fin')
      r = Record(*x)
      run(self.s.update_phase(*x))
    for i in range(fin_max, pre_max):
      x = ((i, 'dflt'), None, 'pre')
      r = Record(*x)
      run(self.s.update_phase(*x))

  def test_update_phase(self):
    """ Test that records can be updated correctly.
    1. Records added should be available.
    2. Another update with identical data should not change anything
    3. Records should be able to update phases 'pre' -> 'fin' -> 'FIN' but never
    any other way.
    """
    s = self.s
    # 1. If we add a record, that record can be found again.
    t1 = (1, 'a')
    w1 = "1-a.elem"
    x = (t1, w1, 'pre')
    r1 = Record(*x)
    run(s.update_phase(*x))
    f1 = s.register[t1]
    self.assertEqual( s.register[t1], r1 )
    # 2. Update with the same values should not change the register.
    run(s.update_phase(*x))
    self.assertEqual( s.register[t1], r1 )
    # 3. Only allow legal phase updates.
    x = (t1, w1, 'fin')
    r2 = Record(*x)
    # 3.1 Update existing record to phase 'fin'
    run(s.update_phase(*x))
    self.assertNotEqual( s.register[t1], r1 )
    self.assertEqual( s.register[t1], r2 )
    # 3.2 Try to change from 'fin' to 'pre', with element != None
    x = (t1, w1, 'pre')
    r3 = Record(*x)
    run(s.update_phase(*x))
    self.assertEqual( s.register[t1], r3 )
    # 3.3 Try to change from 'fin' to 'pre', with element = None
    x = (t1, None, 'pre')
    run(s.update_phase(*x))
    self.assertEqual( s.register[t1].phase, 'pre')
    # 3.4 Try to change from 'pre' to 'FIN', with element != None
    x = (t1, w1, 'pre')
    run(s.update_phase(*x))
    x = (t1, w1, 'FIN')
    run(s.update_phase(*x))
    self.assertEqual( s.register[t1].phase, 'FIN')
    # 3.5 Try to change from 'pre' to 'fin' and then to 'FIN', with element != None
    x = (t1, w1, 'fin')
    run(s.update_phase(*x))
    x = (t1, w1, 'FIN')
    run(s.update_phase(*x))
    self.assertEqual( s.register[t1].phase, 'FIN')

  def test_max_phase(self):
    """ Test that the correct tag is returned for each set of phases. """
    s = self.s
    all_sets = []
    for i in range(len(self.phases)+1):
      all_sets += itertools.combinations(self.phases, i)
    all_sets = [list(a) for a in all_sets][1:]
    for D in all_sets:
      if 'pre' in D:
        self.assertEqual( s.max_phase(D)[0], self.pre_max-1)
      elif 'fin' in D:
        self.assertEqual( s.max_phase(D)[0], self.fin_max-1)
      else:
        self.assertEqual( s.max_phase(D)[0], self.FIN_max-1)

  def test_tag_tuple(self):
    """ Test that the correct Record is returned for each set of phases. """
    tt = self.s.tag_tuple()
    self.assertGreaterEqual(tt[0], tt[1])
    self.assertGreaterEqual(tt[1], tt[2])

if __name__ == '__main__':
  unittest.main()
