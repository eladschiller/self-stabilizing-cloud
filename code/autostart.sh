#!/bin/bash
# Checks if node is server. If it is, it starts the server program.
# Otherwise, it starts the Client application prompt.

address=$(hostname)
slice=chalmersple_casss2
config=./casss/config/autogen.ini

# For each time my address is in autogen.ini, start a server
servers=$(grep $address $config | cut -d ' ' -f 3)

for server in $servers; do
  address=$(echo $server | cut -d ':' -f 1)
  port=$(echo $server | cut -d ':' -f 2)
  if [ $# -eq 0 ]; then
    echo "Starting server at $address:$port"
    sudo python3.6 -O /home/$slice/casss/server.py $port $address $config > /dev/null &
  else
    echo "Starting AR server at $address:$port"
    sudo python3.6 -O /home/$slice/casss/ARserver.py $port $address $config > /dev/null &
  fi
done
