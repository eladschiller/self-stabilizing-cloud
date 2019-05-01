#!/bin/env python3.6

import sys
import os

path = sys.argv[1]
addr_list = []
with open(path) as f:
    addr_list = f.read().splitlines()
avg_list = []
for addr in addr_list:
    response = os.popen("ping -c 10 "+addr+" | tail -1 | awk '{print $4}' | cut -d '/' -f 2").read()
    avg_list.append(float(response))
max_val = round(max(avg_list), 2)
min_val = round(min(avg_list), 2)
avg_val = round(sum(avg_list)/float(len(avg_list)), 2)
print(f"min\t& max\t& avg")
print(f"{min_val}\t& {max_val}\t& {avg_val}")
