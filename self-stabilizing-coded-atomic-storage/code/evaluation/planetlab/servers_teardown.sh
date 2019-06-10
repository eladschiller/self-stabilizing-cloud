#!/bin/bash

opts="StrictHostKeyChecking=no"
slice=chalmersple_casss2
servers=$(cat ./config/servers.txt | sort | uniq)

echo "Killing servers"

for server in $servers; do
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $server \
    "sudo pkill -9 python3.5; sudo pkill -9 python3.6; sudo find ~ -path '*/.storage*' -delete; sudo rm -rf ~/data"
done

echo "DONE"
