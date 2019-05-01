#!/bin/bash

if [ $# -eq 0 ]; then
    nodes=4
else
    nodes=$1
fi

resfile=perf_result.txt
echo "Result of test with $nodes nodes at $(date +'%F_%T')" >> $resfile
echo "==================================================" >> $resfile

echo "Setting up test environment..."
./evaluation/create_new.sh $nodes

n=$(grep -E "^n ="./config/autogen.ini | cut -f 3 -d ' ')
n=$(((n+1)/2))

ips=$(tail -n $n ./config/autogen.ini |\
  cut -d ' ' -f 3 |\
  cut -d ':' -f 1)

# Start NS-3
echo "Starting NS-3..."
pushd ./ns3/repos/ns-3-allinone/ns-3-dev
sudo ./waf --run "tap-csma-virtual-machine --n=$nodes" &
waf=$!
popd

i=$n
for ip in $ips; do
  docker exec c$i iperf3 -sD
  i=$((i+1))
done

pids=()
i=0
for ip in $ips; do
  docker exec c$i iperf3 -c $ip | tee /dev/tty | grep -E "receiver" >> $resfile &
  pids+=("$!")
  i=$((i+1))
done

for p in "${pids[@]}"; do
  wait $p
done

echo "" >> $resfile

echo "Stopping NS-3: $waf"
sudo kill -2 $waf $(pgrep -P $waf)

echo "Tearing down test environment..."
~/thesis-code/evaluation/remove_old.sh $nodes
~/thesis-code/evaluation/remove_old.sh $nodes
~/thesis-code/evaluation/remove_old.sh $nodes
