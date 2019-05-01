#!/bin/bash

opts="StrictHostKeyChecking=no"
slice=chalmersple_casss2
clients=$(cat ./config/clients.txt | sort | uniq)

echo "Removing clients results directory"

for client in $clients; do
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $client \
    "sudo pkill -9 python3.5; sudo pkill -9 python3.6; rm -r ~/results/*"
done

echo "DONE"
