#!/bin/bash

opts="StrictHostKeyChecking=no"

nodes=$(cat $1 | sort | uniq)
slice=$2

for machine in $nodes; do
  echo "$machine"
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $machine \
    "sudo /home/$slice/casss/evaluation/planetlab/ping.py /home/$slice/casss/config/servers.txt"
done
