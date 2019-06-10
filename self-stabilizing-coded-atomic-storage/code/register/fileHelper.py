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

""" Helper functions for file io """

import sys
import os
import pathlib
import aiofiles

default_path = "./.storage/"

async def write_file(data, filename, path=default_path):
  """ Tries to write data to a file.

  Will make sure that the path exists by creating any missing (parent)
  directories. Will always overwrite any existing file. If data is string, it is
  encoded to a byte object (to make it easier to test things ad hoc).

  Args:
    data (bytes): The data to be written
    filename (string): Filename
    path (path, optional): Where to write the file. Default is ./storage/

  Returns:
    string: /path/to/file.txt
  """
  pathlib.Path(path).mkdir(exist_ok=True, parents=True)
  filepath = path + filename
  if type(data) == str:
    data = str.encode(data)
  try:
    async with aiofiles.open(filepath, 'wb') as f:
      await f.write(data)
    return filename
  except OSError as e:
    if __debug__:
      print(f"Error reading file {filepath}: {e}", file=sys.stderr)

async def read_file(filename, path=default_path):
  """ Tries to read (binary) data from a file.

  Args:
    filename (string): /path/to/file.txt
  Returns:
    data (bytes): File content
  """
  filepath = path + filename
  try:
    async with aiofiles.open(filepath, 'rb') as f:
      data = await f.read()
    return data
  except OSError as e:
    if __debug__:
      print(f"Error reading file {path}: {e}", file=sys.stderr)

def delete_file(filename, path=default_path):
  """ Deletes a file.

  Args:
    filename (string): /path/to/file.txt
  """
  filepath = path + filename
  try:
    os.remove(filepath)
  except OSError as e:
    if __debug__:
      print(f"Error deleting file {path}: {e}")#, file=sys.stderr)
