#!/bin/bash

opts="StrictHostKeyChecking=no"
slice=chalmersple_casss2
all_things=$(cat ./config/servers.txt ./config/clients.txt | sort | uniq)

echo "Removing clients results directory"

for a in $all_things; do
  echo "SSH to $a"
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $a "echo done"
done

echo "DONE"
