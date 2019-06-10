#!/bin/bash

opts="StrictHostKeyChecking=no"

if [ $# -eq 0 ]; then
  nodes=$(cat ./servers.txt ./clients.txt | sort | uniq)
  slice=chalmersple_casss2
else
  nodes=$(cat $1 | sort | uniq)
  slice=$2
fi

echo "Checking bind status..."

for machine in $nodes; do
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $machine \
    'echo "$(hostname):"; sudo nc -l $(hostname) -w 3'
done

echo "DONE"
