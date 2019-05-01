#!/bin/python3.6
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
import configparser

def main(filename, nbr_of_servers, f):
    k = nbr_of_servers - 2*f
    if(k < 1):
        raise Exception("Coded elements less than 1")
    config = configparser.ConfigParser()
    config['Nodes'] = {}
    base_port = 5550
    for i in range(nbr_of_servers):
        nodename = 'Node{}'.format(i)
        port = base_port + i
        config['Nodes'][nodename] = '127.0.0.1:{}'.format(port)

    config['General'] = {}
    config['General']['n'] = str(nbr_of_servers)
    config['General']['f'] = str(f)
    config['General']['e'] = '0'
    config['General']['storage_location'] = './.storage/'
    config['General']['max_clients'] = '10'
    config['General']['concurrent_clients'] = '5'
    config['General']['queue_size'] = '10'
    config['General']['gossip_freq'] = '1'
    config['General']['chunks_size'] = '1024'

    with open(filename, 'w') as cfgfile:
        config.write(cfgfile)

if __name__ == '__main__':
    filename = input('Filename: ') + '.ini'
    nbr_of_servers = int(input('Number of servers: '))
    f = int(input('Number of crash prone servers: '))
    main(filename, nbr_of_servers, f)
