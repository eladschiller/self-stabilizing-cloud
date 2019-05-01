#!/bin/bash
# Checks if node is server. If it is, it starts the server program.
# Otherwise, it starts the Client application prompt.

address=$(hostname -i)
slice=chalmersple_casss2
config=./casss/config/autogen.ini

# For each time my address is in autogen.ini, start a server
servers=$(grep $address $config | cut -d ' ' -f 3)

for server in $servers; do
  address=$(echo $server | cut -d ':' -f 1)
  port=$(echo $server | cut -d ':' -f 2)
  echo "Starting server at $address:$port"
  python3.5 -O /home/$slice/casss/start_cas_zmq_servers.py $port $address $config > /dev/null &
done
