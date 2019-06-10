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

from collections import deque

class IncNbrHelper:
    def merge(q, l):
        for hw_addr, incnbr in l:
            if (hw_addr, None) in q:
                index = q.index((hw_addr, None))
                prev = q[index][1].nbr
                q.remove((hw_addr, None))
            else:
                prev = 0
            q.append((hw_addr, IncNbr(max(prev, incnbr))))

    def max_reached(q, m):
        for _, nbr in q:
            if nbr.nbr > m:
                return True
        return False

    def get(q, hw_addr):
        if (hw_addr, None) in q:
            index = q.index((hw_addr, None))
            tmp = q[index]
            q.remove((hw_addr, None))
            q.append(tmp)
            return tmp[1].nbr
        else:
            return 0

    def set(q, hw_addr, newIncNbr):
        if (hw_addr, None) in q:
            q.remove((hw_addr, None))
        q.append((hw_addr, IncNbr(newIncNbr)))

    def to_list(q):
        return [(a, b.nbr) for a, b in q]

class IncNbr:
    def __init__(self, nbr):
        self.nbr = nbr
    def __eq__(self, other):
        if not other:
            return True
        elif isinstance(self, other.__class__):
            return True
        else:
            return False
