#!/bin/bash

opts="StrictHostKeyChecking=no"
ssh_key=~/.ssh/planetlab_rsa
slice=chalmersple_casss2

if [ $# -eq 0 ]; then
  nodes=$(cat ./servers.txt ./clients.txt | sort | uniq)
else
  nodes=$(cat $1 | sort | uniq)
fi

echo "Checking bind status..."

for machine in $nodes; do
  ssh -o $opts -l $slice -i $ssh_key $machine\
  '~/casss/config/util/pyeclib_tester.py && echo "$(hostname) works" || echo "$(hostname) is broken"'
done

echo "DONE"
