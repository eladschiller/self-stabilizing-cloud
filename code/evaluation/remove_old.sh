#!/bin/bash

if [ $# -eq 0 ]; then
    n=4
else
    n=$(($1-1))
fi

# 0. Remove old setup
echo "Removing old setup..."
for i in $( seq 0 $n ); do
    docker container stop "c$i"
    docker container rm "/c$i"
    sudo ip link set "tap-$i" "up"
    sudo ip link set "tap-$i" "down"
    sudo tunctl -d tap-$i
done;
docker network prune -f
echo "Removed all old containers, bridges and taps!"

echo "Removing storage volume..."
docker volume rm storage
echo "Removed storage volume!"
