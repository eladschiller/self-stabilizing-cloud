#!/bin/bash

opts="StrictHostKeyChecking=no"
ssh_key=~/.ssh/planetlab_rsa
slice=chalmersple_casss2
servers=$(cat ./config/servers.txt | sort | uniq)
failed=0

./evaluation/planetlab/setup/create_config.sh
./evaluation/planetlab/setup/copy_to_planetlab.sh

for server in $servers; do
  ssh -o $opts -l $slice -i ~/.ssh/planetlab_rsa $server "~/casss/cas_zmq_autostart.sh" \
    && echo "Running CAS_zmq server on $server"  \
    || { echo "Failed on $server"; failed=1; break; }
done


if [ $failed -eq 0 ]; then
  echo "Service is running"
else
  echo "Service down"
fi
